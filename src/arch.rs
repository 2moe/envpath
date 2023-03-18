/// Returns a string indicating the Debian architecture based on the current target architecture and any additional features.
///
/// # Returns
///
/// A string representing the Debian architecture, e.g. "amd64", "riscv64", "arm64", "ppc64el".
///
/// # Table
///
/// | Architecture                | deb_arch       |
/// | --------------------------- | -------------- |
/// | x86_64                      | amd64          |
/// | aarch64                     | arm64          |
/// | riscv64 (riscv64gc)         | riscv64        |
/// | arm (feature = `+vfpv3`)    | armhf          |
/// | arm                         | armel          |
/// | mips (endian = little)      | mipsel         |
/// | mips64 (endian = little)    | mips64el       |
/// | s390x                       | s390x          |
/// | powerpc64 (endian = little) | ppc64el        |
/// | x86 (i586/i686)             | i386           |
/// | other                       | [consts::ARCH](::std::env::consts::ARCH) |
///
/// # Examples
///
/// ```
/// let deb_arch = envpath::arch::get_deb_arch();
/// println!("Debian architecture: {}", deb_arch);
///
/// #[cfg(target_arch = "x86_64")]
/// assert_eq!("amd64", deb_arch);
///
/// ```
pub const fn get_deb_arch() -> &'static str {
    //    use
    match () {
        #[cfg(target_arch = "x86_64")]
        () => "amd64",

        #[cfg(target_arch = "aarch64")]
        () => "arm64",

        #[cfg(target_arch = "riscv64")]
        () => "riscv64",

        #[cfg(all(target_arch = "arm", target_feature = "vfpv3"))]
        () => "armhf",

        #[cfg(all(target_arch = "arm", not(target_feature = "vfpv3")))]
        () => "armel",

        #[cfg(all(target_arch = "mips", target_endian = "little"))]
        () => "mipsel",

        #[cfg(all(target_arch = "mips64", target_endian = "little"))]
        () => "mips64el",

        #[cfg(target_arch = "s390x")]
        () => "s390x",

        #[cfg(all(target_arch = "powerpc64", target_endian = "little"))]
        () => "ppc64el",

        #[cfg(target_arch = "x86")]
        () => "i386",

        #[allow(unreachable_patterns)]
        _ => std::env::consts::ARCH,
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn print_deb_arch() {
        let arch = super::get_deb_arch();
        dbg!(arch);
    }
}
