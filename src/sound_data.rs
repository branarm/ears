// The MIT License (MIT)
//
// Copyright (c) 2013 Jeremy Letang (letang.jeremy@gmail.com)
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
// FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
// COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
// IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

//! The datas extracted from a sound file.

use std::mem;
use libc::c_void;
use std::vec::Vec;

use openal::{ffi, al};
use sndfile::{SndFile, SndInfo, Read};
use internal::OpenAlData;
use audio_tags::{Tags, AudioTags, get_sound_tags};

/**
 * Samples extracted from a file.
 *
 * SoundDatas are made to be shared between several Sound and played in the same
 * time.
 *
 * # Example
 * ```
 * extern crate ears;
 * use ears::{Sound, SoundData};
 *
 * fn main() -> () {
 *   // Create a SoundData
 *   let snd_data = Rc::new(RefCell::new(SoundData::new(~"path/to/my/sound.wav")
 *                                       .unwrap()));
 *
 *   // Create two Sound with the same SoundData
 *   let snd1 = Sound::new_with_data(snd_data.clone()).unwrap();
 *   let snd2 = Sound::new_with_data(snd_data.clone()).unwrap();
 *
 *   // Play the sounds
 *   snd1.play();
 *   snd2.play();
 *
 *   // Wait until snd2 is playing
 *   while snd2.is_playing() {}
 * }
 * ```
 */
pub struct SoundData {
    /// The SoundTags who contains all the information of the sound
    sound_tags: Tags,
    /// The sndfile samples information
    snd_info: SndInfo,
    /// The total samples count of the Sound
    nb_sample: i64,
    /// The OpenAl internal identifier for the buffer
    al_buffer: u32
}

impl SoundData {
    /**
     * Create a new SoundData.
     *
     * The SoundData contains all the information extracted from the
     * file: samples and tags.
     * It's an easy way to share the same samples between man Sounds objects.
     *
     * # Arguments
     * * `path` - The path of the file to load
     *
     * # Return
     * An Option with Some(SoundData) if the SoundData is create, or None if
     * an error has occured.
     */
    pub fn new(path: &str) -> Option<SoundData> {
        check_openal_context!(None);

        let mut file;

        match SndFile::new(path, Read) {
            Ok(file_) => file = file_,
            Err(err) => { println!("{}", err); return None; }
        };

        let infos = file.get_sndinfo();

        let nb_sample = infos.channels as i64 * infos.frames;

        let mut samples = Vec::from_elem(nb_sample as uint, 0i16);
        file.read_i16(samples.as_mut_slice(), nb_sample as i64);

        let mut buffer_id = 0;
        let len = mem::size_of::<i16>() * (samples.len());

        // Retrieve format informations
        let format =  match al::get_channels_format(infos.channels) {
            Some(fmt) => fmt,
            None => {
                println!("Internal error : unrecognized format.");
                return None;
            }
        };

        al::alGenBuffers(1, &mut buffer_id);
        al::alBufferData(buffer_id,
                         format,
                         samples.as_ptr() as *c_void,
                         len as i32,
                         infos.samplerate);

        match al::openal_has_error() {
            Some(err)   => { println!("{}", err); return None; },
            None        => {}
        };

        let sound_data = SoundData {
            sound_tags  : get_sound_tags(&file),
            snd_info    : infos,
            nb_sample   : nb_sample,
            al_buffer   : buffer_id
        };
        file.close();

        Some(sound_data)
    }
}


/**
 * Get the sound file infos.
 *
 * # Return
 * The struct SndInfo.
 */
pub fn get_sndinfo<'r>(s_data: &'r SoundData) -> &'r SndInfo {
    &s_data.snd_info
}

/**
 * Get the OpenAL identifier of the samples buffer.
 *
 * # Return
 * The OpenAL internal identifier for the samples buffer of the sound.
 */
 #[doc(hidden)]
pub fn get_buffer(s_data: &SoundData) -> u32 {
    s_data.al_buffer
}

impl AudioTags for SoundData {
    /**
     * Get the tags of a Sound.
     *
     * # Return
     * A borrowed pointer to the internal struct SoundTags
     */
    fn get_tags(&self) -> Tags {
        self.sound_tags.clone()
    }
}

impl Drop for SoundData {
    /// Destroy all the resources attached to the SoundData
    fn drop(&mut self) -> () {
        unsafe {
            ffi::alDeleteBuffers(1, &mut self.al_buffer);
        }
    }
}

#[cfg(test)]
mod test {
    #[allow(unused_variable)]
    use sound_data::SoundData;

    #[test]
    fn sounddata_create_OK() -> () {
        #![allow(unused_variable)]
        let snd_data = SoundData::new("shot.wav").unwrap();

    }

    #[test]
    #[should_fail]
    fn sounddata_create_FAIL() -> () {
        #![allow(unused_variable)]
        let snd_data = SoundData::new("toto.wav").unwrap();
    }
}
