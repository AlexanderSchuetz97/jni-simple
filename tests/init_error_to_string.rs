#[cfg(all(feature = "loadjvm", feature = "std"))]
mod test {
    use jni_simple::{LoadFromJavaHomeError, LoadFromJavaHomeFolderError, LoadFromLibraryError};
    use std::io;

    #[test]
    fn test() {
        assert_eq!(format!("{}", LoadFromJavaHomeError::AlreadyLoaded), format!("{}", LoadFromLibraryError::AlreadyLoaded));
        assert_eq!(
            format!(
                "{}",
                LoadFromJavaHomeError::LoadingSharedObjectFailed {
                    path: "".to_string(),
                    error: "".into()
                }
            ),
            format!(
                "{}",
                LoadFromLibraryError::LoadingSharedObjectFailed {
                    path: "".to_string(),
                    error: "".into()
                }
            )
        );
        assert_eq!(
            format!(
                "{}",
                LoadFromJavaHomeError::JNICreateJavaVmNotFound {
                    path: "".to_string(),
                    error: "".into()
                }
            ),
            format!(
                "{}",
                LoadFromLibraryError::JNICreateJavaVmNotFound {
                    path: "".to_string(),
                    error: "".into()
                }
            )
        );
        assert_eq!(
            format!(
                "{}",
                LoadFromJavaHomeError::JNIGetCreatedJavaVMsNotFound {
                    path: "".to_string(),
                    error: "".into()
                }
            ),
            format!(
                "{}",
                LoadFromLibraryError::JNIGetCreatedJavaVMsNotFound {
                    path: "".to_string(),
                    error: "".into()
                }
            )
        );

        assert_eq!(
            format!("{}", LoadFromJavaHomeFolderError::AlreadyLoaded),
            format!("{}", LoadFromLibraryError::AlreadyLoaded)
        );
        assert_eq!(
            format!(
                "{}",
                LoadFromJavaHomeFolderError::LoadingSharedObjectFailed {
                    path: "".to_string(),
                    error: "".into()
                }
            ),
            format!(
                "{}",
                LoadFromLibraryError::LoadingSharedObjectFailed {
                    path: "".to_string(),
                    error: "".into()
                }
            )
        );
        assert_eq!(
            format!(
                "{}",
                LoadFromJavaHomeFolderError::JNICreateJavaVmNotFound {
                    path: "".to_string(),
                    error: "".into()
                }
            ),
            format!(
                "{}",
                LoadFromLibraryError::JNICreateJavaVmNotFound {
                    path: "".to_string(),
                    error: "".into()
                }
            )
        );
        assert_eq!(
            format!(
                "{}",
                LoadFromJavaHomeFolderError::JNIGetCreatedJavaVMsNotFound {
                    path: "".to_string(),
                    error: "".into()
                }
            ),
            format!(
                "{}",
                LoadFromLibraryError::JNIGetCreatedJavaVMsNotFound {
                    path: "".to_string(),
                    error: "".into()
                }
            )
        );

        assert_eq!(
            format!("{}", LoadFromJavaHomeFolderError::UnknownJavaHomeLayout),
            format!("{}", LoadFromJavaHomeError::UnknownJavaHomeLayout)
        );
        assert_eq!(
            format!("{}", LoadFromJavaHomeFolderError::IOError(io::Error::other(""))),
            format!("{}", LoadFromJavaHomeError::IOError(io::Error::other("")))
        );
    }
}
