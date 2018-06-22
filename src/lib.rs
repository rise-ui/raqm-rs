#[macro_use]
extern crate failure;
extern crate raqm_sys;

use std::os::raw::c_int;

#[derive(Debug, Fail)]
pub enum RaqmError {
    #[fail(display = "raqm_create() returned NULL")]
    CreateFailed,
    // TODO: sensible errors if that's possible with libraqm
    #[fail(display = "libraqm error")]
    Failed,
}

pub type Result<T> = ::std::result::Result<T, RaqmError>;

// Import functions
use raqm_sys::{
    raqm_add_font_feature, raqm_create, raqm_destroy, raqm_reference,
    raqm_set_language, raqm_set_par_direction, raqm_set_text, raqm_set_text_utf8,
    raqm_set_freetype_face,
    raqm_set_freetype_face_range,
    raqm_set_freetype_load_flags,
    raqm_layout,
    raqm_get_glyphs,
    raqm_index_to_position,
    raqm_position_to_index,
};

// Import types
use raqm_sys::{
    raqm_t, raqm_direction_t, raqm_glyph_t, FT_Face,
};

macro_rules! check_success {
    ($code:expr) => {
        if $code {
            Ok(())
        } else {
            Err(RaqmError::Failed)
        }
    };
}

pub struct Raqm {
    ptr: *mut raqm_t,
}

impl Raqm {
    /// Creates a new raqm_t with all its internal states initialized to their defaults.
    fn new() -> Result<Self> {
        let ptr: *mut raqm_t = unsafe { raqm_create() };
        if !ptr.is_null() {
            Ok(Raqm { ptr })
        } else {
            Err(RaqmError::CreateFailed)
        }
    }

    /// Adds text to rq to be used for layout. It must be a valid UTF-32 text,
    /// any invalid character will be replaced with U+FFFD.
    /// The text should typically represent a full paragraph,
    /// since doing the layout of chunks of text separately can give improper output.
    pub fn set_text_utf32(&mut self, text: &[u32]) -> Result<()> {
        check_success!(
            unsafe { raqm_set_text(self.ptr, text.as_ptr(), text.len()) }
        )
    }

    /// Same as Raqm::set_text_utf32(), but for text encoded in UTF-8 encoding.
    // TODO: intoduce Text type with builder for faces and lang ranges initialization
    // TODO: through one type + one set_text method
    pub fn set_text(&mut self, text: &str) -> Result<()> {
        check_success!(
            unsafe { raqm_set_text_utf8(self.ptr, text.as_ptr() as *const i8, text.len()) }
        )
    }

    /// Sets the paragraph direction, also known as block direction in CSS.
    /// For horizontal text, this controls the overall direction in the Unicode Bidirectional Algorithm,
    /// so when the text is mainly right-to-left (with or without some left-to-right) text,
    /// then the base direction should be set to RaqmDirection::RightToLeft and vice versa.
    ///
    /// The default is RaqmDirection::Default, which determines the paragraph direction based on the
    /// first character with strong bidi type (see rule P2 in Unicode Bidirectional Algorithm),
    /// which can be good enough for many cases but has problems when a mainly right-to-left paragraph
    /// starts with a left-to-right character and vice versa as the detected paragraph direction will be the wrong one,
    /// or when text does not contain any characters with string bidi types (e.g. only punctuation or numbers)
    /// as this will default to left-to-right paragraph direction.
    ///
    /// For vertical, top-to-bottom text, RaqmDirection::TopToBottom should be used.
    /// Raqm, however, provides limited vertical text support and does not handle rotated horizontal
    /// text in vertical text, instead everything is treated as vertical text.
    pub fn set_par_direction(&mut self, direction: RaqmDirection) -> Result<()> {
        let direction = direction as u32;
        check_success!(
            unsafe { raqm_set_par_direction(self.ptr, direction) }
        )
    }

    /// Sets a BCP47 language code to be used for len -number of characters staring at start.
    /// The start and len are input string array indices (i.e. counting bytes in UTF-8 and scalar values in UTF-32).
    ///
    /// This method can be used repeatedly to set different languages for different parts of the text.
    pub fn set_language(&mut self, lang_code: &str, start: usize, end: usize) -> Result<()> {
        check_success!(
            unsafe { raqm_set_language(self.ptr, lang_code.as_ptr() as *const i8, start, end) }
        )
    }

    /// Sets an FT_Face to be used for all characters in rq
    pub fn set_freetype_face(&mut self, face: FT_Face) -> Result<()> {
        check_success!(
            unsafe { raqm_set_freetype_face(self.ptr, face) }
        )
    }

    /// Sets an FT_Face to be used for len -number of characters staring at start.
    /// The start and len are input string array indices (i.e. counting bytes in UTF-8 and scaler values in UTF-32).
    ///
    /// This method can be used repeatedly to set different faces for different parts of the text.
    /// It is the responsibility of the client to make sure that face ranges cover the whole text.
    pub fn set_freetype_face_range(&mut self, face: FT_Face, start: usize, end: usize) -> Result<()> {
        check_success!(
            unsafe { raqm_set_freetype_face_range(self.ptr, face, start, end) }
        )
    }

    /// Sets the load flags passed to FreeType when loading glyphs, should be the same flags used by
    /// the client when rendering FreeType glyphs.
    //
    /// This requires version of HarfBuzz that has hb_ft_font_set_load_flags(), for older version the flags will be ignored.
    // TODO: make a flags enum/builder, c-style frags in public interface are nightmare
    pub fn set_freetype_load_flags(&mut self, flags: i32) -> Result<()> {
        check_success!(
            unsafe { raqm_set_freetype_load_flags(self.ptr, flags) }
        )
    }

    /// Adds a font feature to be used by the raqm_t during text layout. This is usually used to turn
    /// on optional font features that are not enabled by default, for example dlig or ss01,
    /// but can be also used to turn off default font features.
    ///
    /// feature is string representing a single font feature, in the syntax understood by hb_feature_from_string().
    //
    /// This function can be called repeatedly, new features will be appended to the end of the
    /// features list and can potentially override previous features.
    pub fn add_font_feature(&mut self, feature: &str, len: usize) -> Result<()> {
        check_success!(
            unsafe { raqm_add_font_feature(self.ptr, feature.as_ptr() as *const i8, len as c_int) }
        )
    }

    /// Run the text layout process on rq . This is the main Raqm function where the
    /// Unicode Bidirectional Text algorithm will be applied to the text in rq, text shaping,
    /// and any other part of the layout process.
    pub fn layout(&mut self) -> Result<()> {
        check_success!(
            unsafe { raqm_layout(self.ptr) }
        )
    }
}

pub enum RaqmDirection {
    Default = raqm_sys::raqm_direction_t_RAQM_DIRECTION_DEFAULT as isize,
    RightToLeft = raqm_sys::raqm_direction_t_RAQM_DIRECTION_RTL as isize,
    LeftToRight = raqm_sys::raqm_direction_t_RAQM_DIRECTION_LTR as isize,
    TopToBottom = raqm_sys::raqm_direction_t_RAQM_DIRECTION_TTB as isize,
}

impl Drop for Raqm {
    fn drop(&mut self) {
        unsafe { raqm_destroy(self.ptr) }
    }
}
