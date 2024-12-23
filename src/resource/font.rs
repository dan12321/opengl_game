use std::{collections::HashMap, ptr};

use freetype_sys::*;

// 256 - 32 (first 32 aren't chars) - 1 (DEL at end)
const ALPHABET_SIZE: u8 = 255 - 32;

pub struct FreeType {
    lib: FT_Library,
    faces: HashMap<String, FT_Face>,
}

pub struct Alphabet {
    name: String,
    font_size: u32,
    // ASCII Map to character
    chars: [Option<Character>; 256],
}

#[derive(Clone)]
pub struct Character {
    size: (u32, u32),
    offset: (u32, u32),
    width: u32,
    buffer: Vec<u32>,
}

impl FreeType {
    pub fn new() -> Result<FreeType, FTError> {
        unsafe {
            let mut pointer: FT_Library = ptr::null_mut();
            let error = FT_Init_FreeType(&mut pointer);
            match FTError::from_ft_error(error) {
                Some(e) => Err(e),
                None => Ok(FreeType {
                    lib: pointer,
                    faces: HashMap::new(),
                }),
            }
        }
    }

    pub fn load_alphabet(&self, filename: String, pixel_height: u32) -> Result<Alphabet, FTError> {
        let face = self.get_or_load_face(filename.clone())?;
        unsafe {
            FT_Set_Pixel_Sizes(face, 0, pixel_height);
            let chars = [None; 256];
            for i in 0..ALPHABET_SIZE {
                let c = (i + 32) as char;
                FT_Load_Char(face, c as u64, FT_LOAD_RENDER);
                let character = Character {
                };
                chars[i+32] = Some(character);
            }
            Ok(Alphabet {
                name: filename,
                font_size: pixel_height,
                chars,
            })
        }
    }

    fn get_or_load_face(&mut self, filename: String) -> Result<FT_Face, FTError> {
        if let Some(face) = self.faces.get(&filename) {
            return Ok(*face);
        }

        let face = self.load_face(&filename)?;
        self.faces.insert(filename, face);
        Ok(face)
    }

    fn load_face(&mut self, filename: &str) -> Result<FT_Face, FTError> {
        unsafe {
            let mut face: FT_Face = ptr::null_mut();
            let error = FT_New_Face(self.lib, filename.as_ptr() as *const i8, 0, &mut face);
            match FTError::from_ft_error(error) {
                Some(e) => Err(e),
                None => Ok(face),
            }
        }
    }
}

pub enum FTError {
    CannotOpenResource, // 1
    UnknownFileFormat, // 2
    InvalidFileFormat, // 3
    InvalidVersion, // 4
    LowerModuleVersion, // 5
    InvalidArgument, // 6
    UnimplementedFeature, // 7
    InvalidTable, // 8
    InvalidOffset, // 9
    ArrayTooLarge, // 10
    MissingModule, // 11
    MissingProperty, // 12
    InvalidGlyphIndex, // 16
    InvalidCharacterCode, // 17
    InvalidGlyphFormat, // 18
    CannotRenderGlyph, // 19
    InvalidOutline, // 20
    InvalidComposite, // 21
    TooManyHints, // 22
    InvalidPixelSize, // 23
    InvalidHandle, // 32
    InvalidLibraryHandle, // 33
    InvalidDriverHandle, // 34
    InvalidFaceHandle, // 35
    InvalidSizeHandle, // 36
    InvalidSlotHandle, // 37
    InvalidCharMapHandle, // 38
    InvalidCacheHandle, // 39
    InvalidStreamHandle, // 40
    TooManyDrivers, // 48
    TooManyExtensions, // 49
    OutOfMemory, // 64
    UnlistedObject, // 65
    CannotOpenStream, // 81
    InvalidStreamSeek, // 82
    InvalidStreamSkip, // 83
    InvalidStreamRead, // 84
    InvalidStreamOperation, // 85
    InvalidFrameOperation, // 86
    NestedFrameAccess, // 87
    InvalidFrameRead, // 88
    RasterUninitialized, // 96
    RasterCorrupted, // 97
    RasterOverflow, // 98
    RasterNegativeHeight, // 99
    TooManyCaches, // 112
    InvalidOpcode, // 128
    TooFewArguments, // 129
    StackOverflow, // 130
    CodeOverflow, // 131
    BadArgument, // 132
    DivideByZero, // 133
    InvalidReference, // 134
    DebugOpCode, // 135
    ENDFInExecStream, // 136
    NestedDEFS, // 137
    InvalidCodeRange, // 138
    ExecutionTooLong, // 139
    TooManyFunctionDefs, // 140
    TooManyInstructionDefs, // 141
    TableMissing, // 142
    HorizHeaderMissing, // 143
    LocationsMissing, // 144
    NameTableMissing, // 145
    CMapTableMissing, // 146
    HmtxTableMissing, // 147
    PostTableMissing, // 148
    InvalidHorizMetrics, // 149
    InvalidCharMapFormat, // 150
    InvalidPPem, // 151
    InvalidVertMetrics, // 152
    CouldNotFindContext, // 153
    InvalidPostTableFormat, // 154
    InvalidPostTable, // 155
    SyntaxError, // 160
    StackUnderflow, // 161
    Ignore, // 162
    NoUnicodeGlyphName, // 163
    MissingStartfontField, // 176
    MissingFontField, // 177
    MissingSizeField, // 178
    MissingFontboundingboxField, // 179
    MissingCharsField, // 180
    MissingStartcharField, // 181
    MissingEncodingField, // 182
    MissingBbxField, // 183
    BbxTooBig, // 184
    CorruptedFontHeader, // 185
    CorruptedFontGlyphs, // 186
    Max, // 187
    InvalidErrorCode,
}

impl FTError {
    fn from_ft_error(error: FT_Error) -> Option<Self> {
        match error {
            0 => None,
            1 => Some(Self::CannotOpenResource),
            2 => Some(Self::UnknownFileFormat),
            3 => Some(Self::InvalidFileFormat),
            4 => Some(Self::InvalidVersion),
            5 => Some(Self::LowerModuleVersion),
            6 => Some(Self::InvalidArgument),
            7 => Some(Self::UnimplementedFeature),
            8 => Some(Self::InvalidTable),
            9 => Some(Self::InvalidOffset),
            10 => Some(Self::ArrayTooLarge),
            11 => Some(Self::MissingModule),
            12 => Some(Self::MissingProperty),
            16 => Some(Self::InvalidGlyphIndex),
            17 => Some(Self::InvalidCharacterCode),
            18 => Some(Self::InvalidGlyphFormat),
            19 => Some(Self::CannotRenderGlyph),
            20 => Some(Self::InvalidOutline),
            21 => Some(Self::InvalidComposite),
            22 => Some(Self::TooManyHints),
            23 => Some(Self::InvalidPixelSize),
            32 => Some(Self::InvalidHandle),
            33 => Some(Self::InvalidLibraryHandle),
            34 => Some(Self::InvalidDriverHandle),
            35 => Some(Self::InvalidFaceHandle),
            36 => Some(Self::InvalidSizeHandle),
            37 => Some(Self::InvalidSlotHandle),
            38 => Some(Self::InvalidCharMapHandle),
            39 => Some(Self::InvalidCacheHandle),
            40 => Some(Self::InvalidStreamHandle),
            48 => Some(Self::TooManyDrivers),
            49 => Some(Self::TooManyExtensions),
            64 => Some(Self::OutOfMemory),
            65 => Some(Self::UnlistedObject),
            81 => Some(Self::CannotOpenStream),
            82 => Some(Self::InvalidStreamSeek),
            83 => Some(Self::InvalidStreamSkip),
            84 => Some(Self::InvalidStreamRead),
            85 => Some(Self::InvalidStreamOperation),
            86 => Some(Self::InvalidFrameOperation),
            87 => Some(Self::NestedFrameAccess),
            88 => Some(Self::InvalidFrameRead),
            96 => Some(Self::RasterUninitialized),
            97 => Some(Self::RasterCorrupted),
            98 => Some(Self::RasterOverflow),
            99 => Some(Self::RasterNegativeHeight),
            112 => Some(Self::TooManyCaches),
            128 => Some(Self::InvalidOpcode),
            129 => Some(Self::TooFewArguments),
            130 => Some(Self::StackOverflow),
            131 => Some(Self::CodeOverflow),
            132 => Some(Self::BadArgument),
            133 => Some(Self::DivideByZero),
            134 => Some(Self::InvalidReference),
            135 => Some(Self::DebugOpCode),
            136 => Some(Self::ENDFInExecStream),
            137 => Some(Self::NestedDEFS),
            138 => Some(Self::InvalidCodeRange),
            139 => Some(Self::ExecutionTooLong),
            140 => Some(Self::TooManyFunctionDefs),
            141 => Some(Self::TooManyInstructionDefs),
            142 => Some(Self::TableMissing),
            143 => Some(Self::HorizHeaderMissing),
            144 => Some(Self::LocationsMissing),
            145 => Some(Self::NameTableMissing),
            146 => Some(Self::CMapTableMissing),
            147 => Some(Self::HmtxTableMissing),
            148 => Some(Self::PostTableMissing),
            149 => Some(Self::InvalidHorizMetrics),
            150 => Some(Self::InvalidCharMapFormat),
            151 => Some(Self::InvalidPPem),
            152 => Some(Self::InvalidVertMetrics),
            153 => Some(Self::CouldNotFindContext),
            154 => Some(Self::InvalidPostTableFormat),
            155 => Some(Self::InvalidPostTable),
            160 => Some(Self::SyntaxError),
            161 => Some(Self::StackUnderflow),
            162 => Some(Self::Ignore),
            163 => Some(Self::NoUnicodeGlyphName),
            176 => Some(Self::MissingStartfontField),
            177 => Some(Self::MissingFontField),
            178 => Some(Self::MissingSizeField),
            179 => Some(Self::MissingFontboundingboxField),
            180 => Some(Self::MissingCharsField),
            181 => Some(Self::MissingStartcharField),
            182 => Some(Self::MissingEncodingField),
            183 => Some(Self::MissingBbxField),
            184 => Some(Self::BbxTooBig),
            185 => Some(Self::CorruptedFontHeader),
            186 => Some(Self::CorruptedFontGlyphs),
            187 => Some(Self::Max),
            _ => Some(Self::InvalidErrorCode),
        }
    }
}
