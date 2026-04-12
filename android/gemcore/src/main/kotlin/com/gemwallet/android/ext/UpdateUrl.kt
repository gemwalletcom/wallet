package com.gemwallet.android.ext

private const val UniversalFlavor = "universal"
private const val UniversalApkBaseUrl = "https://apk.gemwallet.com"

fun universalApkDownloadUrl(version: String): String = "$UniversalApkBaseUrl/gem_wallet_universal_v$version.apk"

fun updateUrl(
    flavor: String,
    version: String,
    fallbackUrl: String,
): String = if (flavor == UniversalFlavor) {
    universalApkDownloadUrl(version)
} else {
    fallbackUrl
}
