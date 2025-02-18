use audio_processor_traits::{AudioBuffer, AudioProcessor, Float};
use std::marker::PhantomData;
use std::ops::AddAssign;

/// An `AudioProcessor` which will sum all input channels into input 0.
///
/// If there are no channels it'll no-op. It'll not mute the remaining channels.
pub struct StereoToMonoProcessor<SampleType> {
    phantom: PhantomData<SampleType>,
}

impl<SampleType> Default for StereoToMonoProcessor<SampleType> {
    fn default() -> Self {
        Self::new()
    }
}

impl<SampleType> StereoToMonoProcessor<SampleType> {
    pub fn new() -> Self {
        StereoToMonoProcessor {
            phantom: PhantomData::default(),
        }
    }
}

impl<SampleType> AudioProcessor for StereoToMonoProcessor<SampleType>
where
    SampleType: Float + Sync + Send + AddAssign,
{
    type SampleType = SampleType;

    fn process<BufferType: AudioBuffer<SampleType = Self::SampleType>>(
        &mut self,
        data: &mut BufferType,
    ) {
        if data.num_channels() == 0 {
            return;
        }

        for frame in data.frames_mut() {
            let mut sum: SampleType = SampleType::zero();

            for sample in frame.iter() {
                sum += *sample;
            }

            frame[0] = sum;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use audio_processor_traits::InterleavedAudioBuffer;

    #[test]
    fn test_stereo_to_mono_processor_sums_channels() {
        let mut mono = StereoToMonoProcessor::new();
        let mut samples = [1., 0.1, 1., 0.1, 1., 0.1, 1., 0.1, 1., 0.1, 1., 0.1];
        let mut input = InterleavedAudioBuffer::new(2, &mut samples);

        mono.process(&mut input);

        for sample_index in 0..input.num_samples() {
            let sample = *input.get(0, sample_index);
            assert_eq!(sample, 1.1);
        }
    }

    #[test]
    fn test_stereo_to_mono_can_handle_mono_input() {
        let mut mono = StereoToMonoProcessor::new();
        let mut samples = [1., 1., 1., 1., 1., 1.];
        let mut input = InterleavedAudioBuffer::new(1, &mut samples);

        mono.process(&mut input);

        for sample_index in 0..input.num_samples() {
            let sample = *input.get(0, sample_index);
            assert_eq!(sample, 1.0);
        }
    }

    #[test]
    fn test_stereo_to_mono_can_handle_empty_input() {
        let mut mono = StereoToMonoProcessor::new();
        let mut samples: [f32; 0] = [];
        let mut input = InterleavedAudioBuffer::new(0, &mut samples);

        mono.process(&mut input);
    }
}
