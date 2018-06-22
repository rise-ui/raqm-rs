#[macro_use]
extern crate failure;
extern crate raqm_sys;

#[derive(Debug, Fail)]
pub enum RaqmError {
    #[fail(display = "raqm_create() returned NULL")]
    CreateFailed,
    // TODO: sensible errors if that's possible with libraqm
    #[fail(display = "libraqm error")]
    Failed,
}

pub type Result<T> = ::std::result::Result<T, RaqmError>;

use raqm_sys::{
    raqm_add_font_feature, raqm_create, raqm_destroy, raqm_glyph_t, raqm_reference,
    raqm_set_language, raqm_set_par_direction, raqm_set_text, raqm_set_text_utf8, raqm_t,
    raqm_direction_t,
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
        let direction = direction.into();
        check_success!(
            unsafe { raqm_set_par_direction(self.ptr, direction) }
        )
    }

    //pub fn set_language(&mut self, lang_tag: &str)
}

pub enum RaqmDirection {
    Default,
    RightToLeft,
    LeftToRight,
    TopToBottom
}

impl From<RaqmDirection> for raqm_direction_t {
    fn from(rd: RaqmDirection) -> Self {
        use raqm_sys::{
            raqm_direction_t_RAQM_DIRECTION_DEFAULT,
            raqm_direction_t_RAQM_DIRECTION_RTL,
            raqm_direction_t_RAQM_DIRECTION_LTR,
            raqm_direction_t_RAQM_DIRECTION_TTB,
        };

        match rd {
            RaqmDirection::Default => raqm_direction_t_RAQM_DIRECTION_DEFAULT,
            RaqmDirection::RightToLeft => raqm_direction_t_RAQM_DIRECTION_RTL,
            RaqmDirection::LeftToRight => raqm_direction_t_RAQM_DIRECTION_LTR,
            RaqmDirection::TopToBottom => raqm_direction_t_RAQM_DIRECTION_TTB
        }
    }
}

impl Drop for Raqm {
    fn drop(&mut self) {
        unsafe { raqm_destroy(self.ptr) }
    }
}
