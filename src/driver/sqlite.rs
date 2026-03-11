use crate::error::{DbErr, RuntimeErr};

#[cfg(feature = "sqlite-use-returning-for-3_35")]
pub fn ensure_returning_version(version: &str) -> Result<(), DbErr> {
    let mut parts = version.trim().split('.').map(|part| {
        part.parse::<u32>().map_err(|_| {
            DbErr::Conn(RuntimeErr::Internal(
                "Error parsing SQLite version".to_string(),
            ))
        })
    });

    let mut extract_next = || {
        parts.next().transpose().and_then(|part| {
            part.ok_or_else(|| {
                DbErr::Conn(RuntimeErr::Internal("SQLite version too short".to_string()))
            })
        })
    };

    let major = extract_next()?;
    let minor = extract_next()?;

    if major > 3 || (major == 3 && minor >= 35) {
        Ok(())
    } else {
        Err(DbErr::BackendNotSupported {
            db: "SQLite",
            ctx: "SQLite version does not support returning",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "sqlite-use-returning-for-3_35")]
    #[test]
    fn test_ensure_returning_version() {
        assert!(ensure_returning_version("").is_err());
        assert!(ensure_returning_version(".").is_err());
        assert!(ensure_returning_version(".a").is_err());
        assert!(ensure_returning_version(".4.9").is_err());
        assert!(ensure_returning_version("a").is_err());
        assert!(ensure_returning_version("1.").is_err());
        assert!(ensure_returning_version("1.a").is_err());

        assert!(ensure_returning_version("1.1").is_err());
        assert!(ensure_returning_version("1.0.").is_err());
        assert!(ensure_returning_version("1.0.0").is_err());
        assert!(ensure_returning_version("2.0.0").is_err());
        assert!(ensure_returning_version("3.34.0").is_err());
        assert!(ensure_returning_version("3.34.999").is_err());

        // valid version
        assert!(ensure_returning_version("3.35.0").is_ok());
        assert!(ensure_returning_version("3.35.1").is_ok());
        assert!(ensure_returning_version("3.36.0").is_ok());
        assert!(ensure_returning_version("4.0.0").is_ok());
        assert!(ensure_returning_version("99.0.0").is_ok());
    }
}
