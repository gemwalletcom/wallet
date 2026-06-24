val CHANNELS: Map<String, Map<String, Any?>> = mapOf(
    "google" to mapOf(
        "push" to ":flavors:fcm",
        "review" to ":flavors:google-review",
        "walletConnect" to ":data:services:walletconnect:reown",
        "updateUrl" to "https://play.google.com/store/apps/details?id=com.gemwallet.android",
        "abis" to listOf("armeabi-v7a", "arm64-v8a"),
        "isDefault" to true,
    ),
    "universal" to mapOf(
        "push" to ":flavors:fcm",
        "review" to ":flavors:google-review",
        "walletConnect" to ":data:services:walletconnect:reown",
        "updateUrl" to "https://apk.gemwallet.com/gem_wallet_latest.apk",
        "abis" to listOf("armeabi-v7a", "arm64-v8a"),
        "isDefault" to false,
    ),
    "huawei" to mapOf(
        "push" to ":flavors:pushes-stub",
        "review" to ":flavors:review-stub",
        "walletConnect" to ":data:services:walletconnect:reown",
        "updateUrl" to "https://appgallery.huawei.com/app/C109713129",
        "abis" to listOf("armeabi-v7a", "arm64-v8a"),
        "isDefault" to false,
    ),
    "solana" to mapOf(
        "push" to ":flavors:fcm",
        "review" to ":flavors:review-stub",
        "walletConnect" to ":data:services:walletconnect:reown",
        "updateUrl" to "solanadappstore://details?id=com.gemwallet.android",
        "abis" to listOf("arm64-v8a"),
        "isDefault" to false,
    ),
    "samsung" to mapOf(
        "push" to ":flavors:fcm",
        "review" to ":flavors:review-stub",
        "walletConnect" to ":data:services:walletconnect:reown",
        "updateUrl" to "https://apps.samsung.com/appquery/appDetail.as?appId=com.gemwallet.android",
        "abis" to listOf("armeabi-v7a", "arm64-v8a"),
        "isDefault" to false,
    ),
    "emerald" to mapOf(
        "push" to ":flavors:fcm",
        "review" to ":flavors:review-stub",
        "walletConnect" to ":data:services:walletconnect:reown",
        "updateUrl" to "https://apk.gemwallet.com/gem_wallet_latest.apk",
        "updateUrlEnv" to "UPDATE_URL",
        "abis" to listOf("armeabi-v7a", "arm64-v8a"),
        "isDefault" to false,
    ),
    "fdroid" to mapOf(
        "push" to ":flavors:pushes-stub",
        "review" to ":flavors:review-stub",
        "walletConnect" to ":data:services:walletconnect:noop",
        "updateUrl" to "",
        "abis" to listOf("armeabi-v7a", "arm64-v8a"),
        "isDefault" to false,
    ),
)

fun selectChannel(): String {
    val explicit = gradle.startParameter.projectProperties["channel"]?.takeIf { it.isNotBlank() }
    if (explicit != null) {
        require(CHANNELS.containsKey(explicit)) {
            "Unknown -Pchannel='$explicit'. Known channels: ${CHANNELS.keys}"
        }
        return explicit
    }
    return gradle.startParameter.taskNames
        .asSequence()
        .mapNotNull { task ->
            CHANNELS.keys.firstOrNull { channel ->
                task.contains("assemble$channel", ignoreCase = true) || task.contains("bundle$channel", ignoreCase = true)
            }
        }
        .firstOrNull()
        ?: "google"
}

extra["gemChannels"] = CHANNELS
extra["gemChannel"] = selectChannel()
