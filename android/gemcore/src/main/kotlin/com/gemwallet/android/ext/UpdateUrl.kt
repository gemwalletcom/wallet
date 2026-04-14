package com.gemwallet.android.ext

private const val UniversalFlavor = "universal"
private const val UniversalApkBaseUrl = "https://apk.gemwallet.com"

fun isApkFlavor(flavor: String): Boolean = flavor == UniversalFlavor

fun universalApkDownloadUrl(version: String): String = "$UniversalApkBaseUrl/gem_wallet_universal_$version.apk"

fun updateUrl(
    flavor: String,
    version: String,
    fallbackUrl: String,
): String = if (isApkFlavor(flavor)) {
    universalApkDownloadUrl(version)
} else {
    fallbackUrl
}
