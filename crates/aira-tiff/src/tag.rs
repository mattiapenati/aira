/// The tag of IFD entry.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Tag(pub u16);

impl std::fmt::Debug for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.name(), self.0)
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Tag {
    /* ---------- Baseline TIFF ---------- */
    /// A general indication of the kind of data contained in this subfile.
    pub const NEW_SUBFILE_TYPE: Self = Self(254);
    /// A general indication of the kind of data contained in this subfile.
    pub const SUBFILE_TYPE: Self = Self(255);
    /// The number of columns in the image.
    pub const IMAGE_WIDTH: Self = Self(256);
    /// The number of rows in the image.
    pub const IMAGE_LENGTH: Self = Self(257);
    /// The number of bits per component.
    pub const BITS_PER_SAMPLE: Self = Self(258);
    /// Data compression algorithm.
    pub const COMPRESSION: Self = Self(259);
    /// The color space of the image data.
    pub const PHOTOMETRIC_INTERPRETATION: Self = Self(262);
    /// For black and white TIFF files that represent shades of gray; the technique used to convert from gray to black and white pixels.
    pub const THRESHHOLDING: Self = Self(263);
    /// The width of the dithering or halftoning matrix used to create a dithered or halftoned bilevel file.
    pub const CELL_WIDTH: Self = Self(264);
    /// The length of the dithering or halftoning matrix used to create a dithered or halftoned bilevel file.
    pub const CELL_LENGTH: Self = Self(265);
    /// The logical order of bits within a byte.
    pub const FILL_ORDER: Self = Self(266);
    /// A string that describes the subject of the image.
    pub const IMAGE_DESCRIPTION: Self = Self(270);
    /// The scanner manufacturer.
    pub const MAKE: Self = Self(271);
    /// The scanner model name or number.
    pub const MODEL: Self = Self(272);
    /// For each strip; the byte offset of that strip.
    pub const STRIP_OFFSETS: Self = Self(273);
    /// The orientation of the image with respect to the rows and columns.
    pub const ORIENTATION: Self = Self(274);
    /// The number of components per pixel.
    pub const SAMPLES_PER_PIXEL: Self = Self(277);
    /// The number of rows in each strip.
    pub const ROWS_PER_STRIP: Self = Self(278);
    /// For each strip; the number of bytes in that strip after any compression.
    pub const STRIP_BYTE_COUNTS: Self = Self(279);
    /// The minimum component value used.
    pub const MIN_SAMPLE_VALUE: Self = Self(280);
    /// The maximum component value used.
    pub const MAX_SAMPLE_VALUE: Self = Self(281);
    /// The number of pixels per `ResolutionUnit`` in the `ImageWidth` direction.
    pub const XRESOLUTION: Self = Self(282);
    /// The number of pixels per `ResolutionUnit`` in the `ImageLength` direction.
    pub const YRESOLUTION: Self = Self(283);
    /// How the components of each pixel are stored.
    pub const PLANAR_CONFIGURATION: Self = Self(284);
    /// For each string of contiguous unused bytes in a TIFF file; the byte offset of the string.
    pub const FREE_OFFSETS: Self = Self(288);
    /// For each string of contiguous unused bytes in a TIFF file; the number of bytes in the string.
    pub const FREE_BYTE_COUNTS: Self = Self(289);
    /// The precision of the information contained in the GrayResponseCurve.
    pub const GRAY_RESPONSE_UNIT: Self = Self(290);
    /// For grayscale data; the optical density of each possible pixel value.
    pub const GRAY_RESPONSE_CURVE: Self = Self(291);
    /// The size of the picture represented by an image.
    pub const RESOLUTION_UNIT: Self = Self(296);
    /// Name and version number of the software package(s) used to create the image.
    pub const SOFTWARE: Self = Self(305);
    /// Date and time of image creation.
    pub const DATE_TIME: Self = Self(306);
    /// Person who created the image.
    pub const ARTIST: Self = Self(315);
    /// The computer and/or operating system in use at the time of image creation.
    pub const HOST_COMPUTER: Self = Self(316);
    /// This field defines a Red-Green-Blue color map.
    pub const COLOR_MAP: Self = Self(320);
    /// Description of extra components.
    pub const EXTRA_SAMPLES: Self = Self(338);
    /// Copyright notice.
    pub const COPYRIGHT: Self = Self(33432);

    /* ---------- TIFF extensions ---------- */
    /// The name of the document from which this image was scanned.
    pub const DOCUMENT_NAME: Self = Self(269);
    /// The name of the page from which this image was scanned.
    pub const PAGE_NAME: Self = Self(285);
    /// X position of the image.
    pub const XPOSITION: Self = Self(286);
    /// Y position of the image.
    pub const YPOSITION: Self = Self(287);
    /// TSelf(4)-encoding options.
    pub const T4_OPTIONS: Self = Self(292);
    /// TSelf(6)-encoding options.
    pub const T6_OPTIONS: Self = Self(293);
    /// The page number of the page from which this image was scanned.
    pub const PAGE_NUMBER: Self = Self(297);
    /// Describes a transfer function for the image in tabular style.
    pub const TRANSFER_FUNCTION: Self = Self(301);
    /// LZW predictor.
    pub const PREDICTOR: Self = Self(317);
    /// The chromaticity of the white point of the image.
    pub const WHITE_POINT: Self = Self(318);
    /// The chromaticities of the primaries of the image.
    pub const PRIMARY_CHROMATICITIES: Self = Self(319);
    /// Provides information about how halftoning should be applied to the image.
    pub const HALFTONE_HINTS: Self = Self(321);
    /// The tile width in pixels.
    pub const TILE_WIDTH: Self = Self(322);
    /// The tile length (height) in pixels.
    pub const TILE_LENGTH: Self = Self(323);
    /// The byte offset of each tile.
    pub const TILE_OFFSETS: Self = Self(324);
    /// The number of (compressed) bytes in each tile.
    pub const TILE_BYTE_COUNTS: Self = Self(325);
    /// The set of inks used in a separated image.
    pub const INK_SET: Self = Self(332);
    /// The name of each ink used in a separated image.
    pub const INK_NAMES: Self = Self(333);
    /// The number of inks.
    pub const NUMBER_OF_INKS: Self = Self(334);
    /// The component values that correspond to a Self(0)% dot and Self(100)% dot.
    pub const DOT_RANGE: Self = Self(336);
    /// A description of the printing environment for which this separation is intended.
    pub const TARGET_PRINTER: Self = Self(337);
    /// This field specifies how to interpret each data sample in a pixel.
    pub const SAMPLE_FORMAT: Self = Self(339);
    /// This field specifies the minimum sample value.
    pub const SMIN_SAMPLE_VALUE: Self = Self(340);
    /// This field specifies the maximum sample value.
    pub const SMAX_SAMPLE_VALUE: Self = Self(341);
    /// Expands the range of the `TransferFunction`.
    pub const TRANSFER_RANGE: Self = Self(342);
    /// The JPEG process used to produce the compressed data.
    pub const JPEG_PROC: Self = Self(512);
    /// This field indicates whether a JPEG interchange format bitstream is present in the TIFF file.
    pub const JPEG_INTERCHANGE_FORMAT: Self = Self(513);
    /// The length in bytes of the JPEG interchange format bitstream.
    pub const JPEG_INTERCHANGE_FORMAT_LENGTH: Self = Self(514);
    /// The length of the restart interval used in the compressed image data.
    pub const JPEG_RESTART_INTERVAL: Self = Self(515);
    /// A list of lossless predictor-selection values; one per component.
    pub const JPEG_LOSSLESS_PREDICTORS: Self = Self(517);
    /// A list of point transform values; one per component.
    pub const JPEG_POINT_TRANSFORMS: Self = Self(518);
    /// A list of point transform values; one per component.
    pub const JPEG_QTABLES: Self = Self(519);
    /// A  list of offsets to the DC Huffman tables or the lossless Huffman tables; one per component.
    pub const JPEG_DCTABLES: Self = Self(520);
    /// A list of offsets to the Huffman AC tables; one per component.
    pub const JPEG_ACTABLES: Self = Self(521);
    /// The transformation from RGB to YCbCr image data.
    pub const YCBCR_COEFFICIENTS: Self = Self(529);
    /// Specifies the subsampling factors used for the chrominance components of a YCbCr image
    pub const YCBCR_SUB_SAMPLING: Self = Self(530);
    /// Specifies the positioning of subsampled chrominance components relative to luminance samples.
    pub const YCBCR_POSITIONING: Self = Self(531);
    /// Specifies a pair of headroom and footroom image data values for each pixel component.
    pub const REFERENCE_BLACK_WHITE: Self = Self(532);

    /* ---------- Adobe PageMaker 6.0 ---------- */
    /// A list of offsets to the sub-IFDs; one per component.
    pub const SUBIFDS: Self = Self(330);
    /// Description of the clipping path.
    pub const CLIP_PATH: Self = Self(343);
    /// The number of units that span the width of the image, in terms of integer ClipPath coordinates.
    pub const X_CLIP_PATH_UNITS: Self = Self(344);
    /// The number of units that span the height of the image, in terms of integer ClipPath coordinates.
    pub const Y_CLIP_PATH_UNITS: Self = Self(345);

    /* ---------- GeoTIFF ---------- */
    /// Transformation between raster space and model space: scaling parameters.
    pub const MODEL_PIXEL_SCALE: Self = Self(33550);
    /// Transformation between raster space and model space: tiepoints.
    pub const MODEL_TIEPOINT: Self = Self(33922);
    /// Transformation between raster space and model space: matrix representation.
    pub const MODEL_TRANSFORMATION: Self = Self(34264);
    /// The set of keys of the projection parameters.
    pub const GEO_KEY_DIRECTORY: Self = Self(34735);
    /// The set of double values of the projection parameters.
    pub const GEO_DOUBLE_PARAMS: Self = Self(34736);
    /// The set of string values of the projection parameters.
    pub const GEO_ASCII_PARAMS: Self = Self(34737);

    /* ---------- GDAL ---------- */
    /// GDAL non standard metadata.
    pub const GDAL_METADATA: Self = Self(42112);
    /// GDAL band nodata value.
    pub const GDAL_NO_DATA: Self = Self(42113);
    /// The full set of NITF RPCSelf(00)B values.
    pub const RPCCOEFFICIENT: Self = Self(50844);

    /* ---------- EXIF ---------- */
    /// Exposure time; given in seconds.
    pub const EXPOSURE_TIME: Self = Self(33434);
    /// The F number.
    pub const FNUMBER: Self = Self(33437);
    /// The class of the program used by the camera to set exposure when the picture is taken.
    pub const EXPOSURE_PROGRAM: Self = Self(34850);
    /// Indicates the spectral sensitivity of each channel of the camera used.
    pub const SPECTRAL_SENSITIVITY: Self = Self(34852);
    /// Indicates the ISO Speed and ISO Latitude of the camera or input device as specified in ISO Self(12232).
    pub const ISO_SPEED_RATINGS: Self = Self(34855);
    /// Indicates the Opto-Electric Conversion Function (OECF) specified in ISO Self(14524).
    pub const OECF: Self = Self(34856);
    /// The version of the supported Exif standard.
    pub const EXIF_VERSION: Self = Self(36864);
    /// The date and time when the original image data was generated.
    pub const DATE_TIME_ORIGINAL: Self = Self(36867);
    /// The date and time when the image was stored as digital data.
    pub const DATE_TIME_DIGITIZED: Self = Self(36868);
    /// Specific to compressed data; specifies the channels and complements PhotometricInterpretation
    pub const COMPONENTS_CONFIGURATION: Self = Self(37121);
    /// Specific to compressed data; states the compressed bits per pixel.
    pub const COMPRESSED_BITS_PER_PIXEL: Self = Self(37122);
    /// Shutter speed.
    pub const SHUTTER_SPEED_VALUE: Self = Self(37377);
    /// The lens aperture.
    pub const APERTURE_VALUE: Self = Self(37378);
    /// The value of brightness.
    pub const BRIGHTNESS_VALUE: Self = Self(37379);
    /// The exposure bias.
    pub const EXPOSURE_BIAS_VALUE: Self = Self(37380);
    /// The smallest F number of the lens.
    pub const MAX_APERTURE_VALUE: Self = Self(37381);
    /// The distance to the subject; given in meters.
    pub const SUBJECT_DISTANCE: Self = Self(37382);
    /// The metering mode.
    pub const METERING_MODE: Self = Self(37383);
    /// The kind of light source.
    pub const LIGHT_SOURCE: Self = Self(37384);
    /// Indicates the status of flash when the image was shot.
    pub const FLASH: Self = Self(37385);
    /// The actual focal length of the lens; in mm.
    pub const FOCAL_LENGTH: Self = Self(37386);
    /// Indicates the location and area of the main subject in the overall scene.
    pub const SUBJECT_AREA: Self = Self(37396);
    /// Manufacturer specific information.
    pub const MAKER_NOTE: Self = Self(37500);
    /// Keywords or comments on the image; complements ImageDescription.
    pub const USER_COMMENT: Self = Self(37510);
    /// A tag used to record fractions of seconds for the DateTime tag.
    pub const SUBSEC_TIME: Self = Self(37520);
    /// A tag used to record fractions of seconds for the DateTimeOriginal tag.
    pub const SUBSEC_TIME_ORIGINAL: Self = Self(37521);
    /// A tag used to record fractions of seconds for the DateTimeDigitized tag.
    pub const SUBSEC_TIME_DIGITIZED: Self = Self(37522);
    /// The Flashpix format version supported by a FPXR file.
    pub const FLASHPIX_VERSION: Self = Self(40960);
    /// The color space information tag is always recorded as the color space specifier.
    pub const COLOR_SPACE: Self = Self(40961);
    /// Specific to compressed data; the valid width of the meaningful image.
    pub const PIXEL_XDIMENSION: Self = Self(40962);
    /// Specific to compressed data; the valid height of the meaningful image.
    pub const PIXEL_YDIMENSION: Self = Self(40963);
    /// Used to record the name of an audio file related to the image data.
    pub const RELATED_SOUND_FILE: Self = Self(40964);
    /// Indicates the strobe energy at the time the image is captured; as measured in Beam Candle Power Seconds
    pub const FLASH_ENERGY: Self = Self(41483);
    /// Records the camera or input device spatial frequency table and SFR values in the direction of image width; image height; and diagonal direction; as specified in ISO Self(12233).
    pub const SPATIAL_FREQUENCY_RESPONSE: Self = Self(41484);
    /// Indicates the number of pixels in the image width (X) direction per FocalPlaneResolutionUnit on the camera focal plane.
    pub const FOCAL_PLANE_XRESOLUTION: Self = Self(41486);
    /// Indicates the number of pixels in the image height (Y) direction per FocalPlaneResolutionUnit on the camera focal plane.
    pub const FOCAL_PLANE_YRESOLUTION: Self = Self(41487);
    /// Indicates the unit for measuring FocalPlaneXResolution and FocalPlaneYResolution.
    pub const FOCAL_PLANE_RESOLUTION_UNIT: Self = Self(41488);
    /// Indicates the location of the main subject in the scene.
    pub const SUBJECT_LOCATION: Self = Self(41492);
    /// Indicates the exposure index selected on the camera or input device at the time the image is captured.
    pub const EXPOSURE_INDEX: Self = Self(41493);
    /// Indicates the image sensor type on the camera or input device.
    pub const SENSING_METHOD: Self = Self(41495);
    /// Indicates the image source.
    pub const FILE_SOURCE: Self = Self(41728);
    /// Indicates the type of scene.
    pub const SCENE_TYPE: Self = Self(41729);
    /// Indicates the color filter array (CFA) geometric pattern of the image sensor when a one-chip color area sensor is used.
    pub const CFA_PATTERN: Self = Self(41730);
    /// Indicates the use of special processing on image data; such as rendering geared to output.
    pub const CUSTOM_RENDERED: Self = Self(41985);
    /// Indicates the exposure mode set when the image was shot.
    pub const EXPOSURE_MODE: Self = Self(41986);
    /// Indicates the white balance mode set when the image was shot.
    pub const WHITE_BALANCE: Self = Self(41987);
    /// Indicates the digital zoom ratio when the image was shot.
    pub const DIGITAL_ZOOM_RATIO: Self = Self(41988);
    /// Indicates the equivalent focal length assuming a Self(35)mm film camera; in mm.
    pub const FOCAL_LENGTH_IN35MM_FILM: Self = Self(41989);
    /// Indicates the type of scene that was shot.
    pub const SCENE_CAPTURE_TYPE: Self = Self(41990);
    /// Indicates the degree of overall image gain adjustment.
    pub const GAIN_COLOR: Self = Self(41991);
    /// Indicates the direction of contrast processing applied by the camera when the image was shot.
    pub const CONTRAST: Self = Self(41992);
    /// Indicates the direction of saturation processing applied by the camera when the image was shot.
    pub const SATURATION: Self = Self(41993);
    /// Indicates the direction of sharpness processing applied by the camera when the image was shot.
    pub const SHARPNESS: Self = Self(41994);
    /// This tag indicates information on the picture-taking conditions of a particular camera model.
    pub const DEVICE_SETTING_DESCRIPTION: Self = Self(41995);
    /// Indicates the distance to the subject.
    pub const SUBJECT_DISTANCE_RANGE: Self = Self(41996);
    /// Indicates an identifier assigned uniquely to each image.
    pub const IMAGE_UNIQUE_ID: Self = Self(42016);
}

impl Tag {
    /// Returns the name of the tag if known, otherwise "Unknown" is returned.
    fn name(&self) -> &'static str {
        match *self {
            /* ---------- Baseline TIFF ---------- */
            Self::NEW_SUBFILE_TYPE => "NewSubfileType",
            Self::SUBFILE_TYPE => "SubfileType",
            Self::IMAGE_WIDTH => "ImageWidth",
            Self::IMAGE_LENGTH => "ImageLength",
            Self::BITS_PER_SAMPLE => "BitsPerSample",
            Self::COMPRESSION => "Compression",
            Self::PHOTOMETRIC_INTERPRETATION => "PhotometricInterpretation",
            Self::THRESHHOLDING => "Threshholding",
            Self::CELL_WIDTH => "CellWidth",
            Self::CELL_LENGTH => "CellLength",
            Self::FILL_ORDER => "FillOrder",
            Self::IMAGE_DESCRIPTION => "ImageDescription",
            Self::MAKE => "Make",
            Self::MODEL => "Model",
            Self::STRIP_OFFSETS => "StripOffsets",
            Self::ORIENTATION => "Orientation",
            Self::SAMPLES_PER_PIXEL => "SamplesPerPixel",
            Self::ROWS_PER_STRIP => "RowsPerStrip",
            Self::STRIP_BYTE_COUNTS => "StripByteCounts",
            Self::MIN_SAMPLE_VALUE => "MinSampleValue",
            Self::MAX_SAMPLE_VALUE => "MaxSampleValue",
            Self::XRESOLUTION => "XResolution",
            Self::YRESOLUTION => "YResolution",
            Self::PLANAR_CONFIGURATION => "PlanarConfiguration",
            Self::FREE_OFFSETS => "FreeOffsets",
            Self::FREE_BYTE_COUNTS => "FreeByteCounts",
            Self::GRAY_RESPONSE_UNIT => "GrayResponseUnit",
            Self::GRAY_RESPONSE_CURVE => "GrayResponseCurve",
            Self::RESOLUTION_UNIT => "ResolutionUnit",
            Self::SOFTWARE => "Software",
            Self::DATE_TIME => "DateTime",
            Self::ARTIST => "Artist",
            Self::HOST_COMPUTER => "HostComputer",
            Self::COLOR_MAP => "ColorMap",
            Self::EXTRA_SAMPLES => "ExtraSamples",
            Self::COPYRIGHT => "Copyright",
            /* ---------- TIFF extensions ---------- */
            Self::DOCUMENT_NAME => "DocumentName",
            Self::PAGE_NAME => "PageName",
            Self::XPOSITION => "XPosition",
            Self::YPOSITION => "YPosition",
            Self::T4_OPTIONS => "T4Options",
            Self::T6_OPTIONS => "T6Options",
            Self::PAGE_NUMBER => "PageNumber",
            Self::TRANSFER_FUNCTION => "TransferFunction",
            Self::PREDICTOR => "Predictor",
            Self::WHITE_POINT => "WhitePoint",
            Self::PRIMARY_CHROMATICITIES => "PrimaryChromaticities",
            Self::HALFTONE_HINTS => "HalftoneHints",
            Self::TILE_WIDTH => "TileWidth",
            Self::TILE_LENGTH => "TileLength",
            Self::TILE_OFFSETS => "TileOffsets",
            Self::TILE_BYTE_COUNTS => "TileByteCounts",
            Self::INK_SET => "InkSet",
            Self::INK_NAMES => "InkNames",
            Self::NUMBER_OF_INKS => "NumberOfInks",
            Self::DOT_RANGE => "DotRange",
            Self::TARGET_PRINTER => "TargetPrinter",
            Self::SAMPLE_FORMAT => "SampleFormat",
            Self::SMIN_SAMPLE_VALUE => "SMinSampleValue",
            Self::SMAX_SAMPLE_VALUE => "SMaxSampleValue",
            Self::TRANSFER_RANGE => "TransferRange",
            Self::JPEG_PROC => "JPEGProc",
            Self::JPEG_INTERCHANGE_FORMAT => "JPEGInterchangeFormat",
            Self::JPEG_INTERCHANGE_FORMAT_LENGTH => "JPEGInterchangeFormatLength",
            Self::JPEG_RESTART_INTERVAL => "JPEGRestartInterval",
            Self::JPEG_LOSSLESS_PREDICTORS => "JPEGLosslessPredictors",
            Self::JPEG_POINT_TRANSFORMS => "JPEGPointTransforms",
            Self::JPEG_QTABLES => "JPEGQTables",
            Self::JPEG_DCTABLES => "JPEGDCTables",
            Self::JPEG_ACTABLES => "JPEGACTables",
            Self::YCBCR_COEFFICIENTS => "YCbCrCoefficients",
            Self::YCBCR_SUB_SAMPLING => "YCbCrSubSampling",
            Self::YCBCR_POSITIONING => "YCbCrPositioning",
            Self::REFERENCE_BLACK_WHITE => "ReferenceBlackWhite",
            /* ---------- Adobe PageMaker 6.0 ---------- */
            Self::SUBIFDS => "SubIFDs",
            Self::CLIP_PATH => "ClipPath",
            Self::X_CLIP_PATH_UNITS => "XClipPathUnits",
            Self::Y_CLIP_PATH_UNITS => "YClipPathUnits",
            /* ---------- GeoTIFF ---------- */
            Self::MODEL_PIXEL_SCALE => "ModelPixelScale",
            Self::MODEL_TIEPOINT => "ModelTiepoint",
            Self::MODEL_TRANSFORMATION => "ModelTransformation",
            Self::GEO_KEY_DIRECTORY => "GeoKeyDirectory",
            Self::GEO_DOUBLE_PARAMS => "GeoDoubleParams",
            Self::GEO_ASCII_PARAMS => "GeoAsciiParams",
            /* ---------- GDAL ---------- */
            Self::GDAL_METADATA => "GdalMetadata",
            Self::GDAL_NO_DATA => "GdalNoData",
            Self::RPCCOEFFICIENT => "RpcCoefficient",
            /* ---------- EXIF ---------- */
            Self::EXPOSURE_TIME => "ExposureTime",
            Self::FNUMBER => "FNumber",
            Self::EXPOSURE_PROGRAM => "ExposureProgram",
            Self::SPECTRAL_SENSITIVITY => "SpectralSensitivity",
            Self::ISO_SPEED_RATINGS => "IsoSpeedRatings",
            Self::OECF => "Oecf",
            Self::EXIF_VERSION => "ExifVersion",
            Self::DATE_TIME_ORIGINAL => "DateTimeOriginal",
            Self::DATE_TIME_DIGITIZED => "DateTimeDigitized",
            Self::COMPONENTS_CONFIGURATION => "ComponentsConfiguration",
            Self::COMPRESSED_BITS_PER_PIXEL => "CompressedBitsPerPixel",
            Self::SHUTTER_SPEED_VALUE => "ShutterSpeedValue",
            Self::APERTURE_VALUE => "ApertureValue",
            Self::BRIGHTNESS_VALUE => "BrightnessValue",
            Self::EXPOSURE_BIAS_VALUE => "ExposureBiasValue",
            Self::MAX_APERTURE_VALUE => "MaxApertureValue",
            Self::SUBJECT_DISTANCE => "SubjectDistance",
            Self::METERING_MODE => "MeteringMode",
            Self::LIGHT_SOURCE => "LightSource",
            Self::FLASH => "Flash",
            Self::FOCAL_LENGTH => "FocalLength",
            Self::SUBJECT_AREA => "SubjectArea",
            Self::MAKER_NOTE => "MakerNote",
            Self::USER_COMMENT => "UserComment",
            Self::SUBSEC_TIME => "SubSecTime",
            Self::SUBSEC_TIME_ORIGINAL => "SubSecTimeOriginal",
            Self::SUBSEC_TIME_DIGITIZED => "SubSecTimeDigitized",
            Self::FLASHPIX_VERSION => "FlashpixVersion",
            Self::COLOR_SPACE => "ColorSpace",
            Self::PIXEL_XDIMENSION => "PixelXDimension",
            Self::PIXEL_YDIMENSION => "PixelYDimension",
            Self::RELATED_SOUND_FILE => "RelatedSoundFile",
            Self::FLASH_ENERGY => "FlashEnergy",
            Self::SPATIAL_FREQUENCY_RESPONSE => "SpatialFrequencyResponse",
            Self::FOCAL_PLANE_XRESOLUTION => "FocalPlaneXResolution",
            Self::FOCAL_PLANE_YRESOLUTION => "FocalPlaneYResolution",
            Self::FOCAL_PLANE_RESOLUTION_UNIT => "FocalPlaneResolutionUnit",
            Self::SUBJECT_LOCATION => "SubjectLocation",
            Self::EXPOSURE_INDEX => "ExposureIndex",
            Self::SENSING_METHOD => "SensingMethod",
            Self::FILE_SOURCE => "FileSource",
            Self::SCENE_TYPE => "SceneType",
            Self::CFA_PATTERN => "CfaPattern",
            Self::CUSTOM_RENDERED => "CustomRendered",
            Self::EXPOSURE_MODE => "ExposureMode",
            Self::WHITE_BALANCE => "WhiteBalance",
            Self::DIGITAL_ZOOM_RATIO => "DigitalZoomRatio",
            Self::FOCAL_LENGTH_IN35MM_FILM => "FocalLengthIn35mmFilm",
            Self::SCENE_CAPTURE_TYPE => "SceneCaptureType",
            Self::GAIN_COLOR => "GainColor",
            Self::CONTRAST => "Contrast",
            Self::SATURATION => "Saturation",
            Self::SHARPNESS => "Sharpness",
            Self::DEVICE_SETTING_DESCRIPTION => "DeviceSettingDescription",
            Self::SUBJECT_DISTANCE_RANGE => "SubjectDistanceRange",
            Self::IMAGE_UNIQUE_ID => "ImageUniqueId",
            /* ---------- Unknown ---------- */
            _ => "Unknown",
        }
    }
}
