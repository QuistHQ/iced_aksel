use std::{collections::VecDeque, sync::Arc};

use cpal::{
    DeviceDescription, DeviceType, Sample, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use realfft::RealFftPlanner;
use triple_buffer::triple_buffer;

use crate::{FFT_GAIN_CORRECTION, FFT_SIZE, HOP_SIZE, MAX_BUFFER_SIZE, math};

pub fn enumerate_devices(host: &cpal::Host) -> Vec<DeviceDescription> {
    host.devices()
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|device| device.description().ok())
        .collect()
}

#[expect(clippy::type_complexity)]
pub fn setup_capture_with_device(
    device_name: Option<String>,
) -> (
    Option<cpal::Stream>,
    Option<DeviceDescription>,
    Option<StreamConfig>,
    triple_buffer::Output<Box<[f32]>>,
) {
    let host = cpal::default_host();

    let device = device_name.map_or_else(
        || find_monitor_device(&host).or_else(|| host.default_output_device()),
        |name| find_device_by_name(&host, &name),
    );

    let (input, output) = triple_buffer(&vec![0.0; FFT_SIZE / 2 + 1].into_boxed_slice());

    let device = match device {
        Some(d) => d,
        None => {
            eprintln!("No audio device available");
            return (None, None, None, output);
        }
    };

    let device_info = device.description().ok();
    println!("Using device: {:?}", device_info);

    let (stream, config) = setup_audio_capture_for_device(input, &device);

    (stream, device_info, config, output)
}

fn find_device_by_name(host: &cpal::Host, target_name: &str) -> Option<cpal::Device> {
    host.devices().ok().into_iter().flatten().find(|device| {
        device
            .description()
            .is_ok_and(|description| description.name() == target_name)
    })
}

fn find_monitor_device(host: &cpal::Host) -> Option<cpal::Device> {
    if let Ok(device_name) = std::env::var("AUDIO_DEVICE") {
        println!("Looking for AUDIO_DEVICE override: {}", device_name);
        if let Ok(devices) = host.devices() {
            for device in devices {
                if let Ok(description) = device.description()
                    && let name = description.name()
                    && name.contains(&device_name)
                {
                    println!("Found override device: {}", name);
                    return Some(device);
                }
            }
        }
        eprintln!("Could not find device matching: {}", device_name);
    }

    println!("\n=== Available INPUT devices ===");
    let devices = match host.devices() {
        Ok(devices) => devices,
        Err(e) => {
            eprintln!("Failed to enumerate input devices: {}", e);
            return None;
        }
    };

    let mut pipewire_device = None;
    let mut monitor_device = None;
    let default_device = host.default_input_device();

    for (i, device) in devices.enumerate() {
        if let Ok(description) = device.description() {
            println!("  [{}] INPUT: {}", i, description.name());

            if description.device_type() == DeviceType::Virtual && monitor_device.is_none() {
                monitor_device = Some(device);
                continue;
            }

            if description.driver() == Some("pipewire") && pipewire_device.is_none() {
                pipewire_device = Some(device);
                continue;
            }
        }
    }

    println!("\n=== Available OUTPUT devices ===");
    if let Ok(devices) = host.output_devices() {
        for (i, device) in devices.enumerate() {
            if let Ok(description) = device.description() {
                let name = description.name();
                println!("  [{}] OUTPUT: {}", i, name);
            }
        }
    }

    println!("\nTip: Set AUDIO_DEVICE env var to select a specific device");
    println!("Example: AUDIO_DEVICE=pipewire cargo run --example chart_spectrum\n");

    monitor_device.or(pipewire_device).or(default_device)
}

fn setup_audio_capture_for_device(
    input: triple_buffer::Input<Box<[f32]>>,
    device: &cpal::Device,
) -> (Option<cpal::Stream>, Option<StreamConfig>) {
    println!(
        "Setting up audio capture for device: {}",
        device
            .description()
            .map(|description| description.name().to_owned())
            .unwrap_or_default()
    );

    let supported_config = match device.default_input_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to get default input config: {}", e);
            return (None, None);
        }
    };
    let config = StreamConfig::from(supported_config.clone());

    let mut planner = RealFftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(FFT_SIZE);

    let stream = match supported_config.sample_format() {
        cpal::SampleFormat::F32 => build_input_stream::<f32>(device, &config, input, fft),
        cpal::SampleFormat::I16 => build_input_stream::<i16>(device, &config, input, fft),
        cpal::SampleFormat::U16 => build_input_stream::<u16>(device, &config, input, fft),
        _ => {
            eprintln!(
                "Unsupported sample format: {:?}",
                supported_config.sample_format()
            );
            return (None, None);
        }
    };

    match stream {
        Ok(stream) => {
            if let Err(e) = stream.play() {
                eprintln!("Failed to start stream: {}", e);
                (None, None)
            } else {
                println!("Audio capture running at {} Hz", config.sample_rate);
                (Some(stream), Some(config))
            }
        }
        Err(e) => {
            eprintln!("Failed to build input stream: {}", e);
            (None, None)
        }
    }
}

fn build_input_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut input: triple_buffer::Input<Box<[f32]>>,
    fft: Arc<dyn realfft::RealToComplex<f32>>,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: cpal::Sample + cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    let mut buffer = VecDeque::with_capacity(MAX_BUFFER_SIZE);
    let mut block = vec![0.0f32; FFT_SIZE].into_boxed_slice();
    let mut magnitudes = vec![0.0f32; FFT_SIZE / 2 + 1].into_boxed_slice();

    let mut scratch = fft.make_scratch_vec().into_boxed_slice();
    let mut spectrum = fft.make_output_vec().into_boxed_slice();

    let channels = config.channels as usize;

    device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            let len_after = buffer.len() + data.len() / channels;
            buffer.drain(..len_after.saturating_sub(MAX_BUFFER_SIZE));

            for frame in data.chunks_exact(channels) {
                let mono =
                    frame.iter().copied().map(f32::from_sample).sum::<f32>() / channels as f32;
                buffer.push_back(mono);
            }

            while buffer.len() >= FFT_SIZE {
                for (block, buffer) in block.iter_mut().zip(&buffer) {
                    *block = *buffer;
                }
                math::apply_blackman_harris(&mut block);

                if fft
                    .process_with_scratch(&mut block, &mut spectrum, &mut scratch)
                    .is_ok()
                {
                    for (i, value) in spectrum.iter().enumerate().take(magnitudes.len()) {
                        let mut amplitude = value.norm() * FFT_GAIN_CORRECTION / FFT_SIZE as f32;
                        if i == 0 || i == magnitudes.len() - 1 {
                            amplitude *= 0.5;
                        }
                        magnitudes[i] = amplitude;
                    }

                    let shared = input.input_buffer_mut();
                    for (slot, mag) in shared.iter_mut().zip(magnitudes.iter()) {
                        *slot = *mag;
                    }
                    input.publish();
                }

                buffer.drain(..HOP_SIZE.min(buffer.len()));
            }
        },
        |err| eprintln!("Error on audio stream: {}", err),
        None,
    )
}
