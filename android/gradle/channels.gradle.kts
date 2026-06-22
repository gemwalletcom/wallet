import org.gradle.kotlin.dsl.extra

val CHANNELS: Map<String, Map<String, Any?>> = mapOf(
    "google" to mapOf(
        "push" to ":flavors:fcm",
        "review" to ":flavors:google-review",
        "walletConnect" to true,
        "firebase" to true,
        "googleServices" to true,
        "dependenciesInfo" to true,
        "updateUrl" to "https://play.google.com/store/apps/details?id=com.gemwallet.android",
        "updateUrlEnv" to null,
        "abis" to listOf("armeabi-v7a", "arm64-v8a"),
        "isDefault" to true,
    ),
    "universal" to mapOf(
        "push" to ":flavors:fcm",
        "review" to ":flavors:google-review",
        "walletConnect" to true,
        "firebase" to true,
        "googleServices" to true,
        "dependenciesInfo" to true,
        "updateUrl" to "https://apk.gemwallet.com/gem_wallet_latest.apk",
        "updateUrlEnv" to null,
        "abis" to listOf("armeabi-v7a", "arm64-v8a"),
        "isDefault" to false,
    ),
    "huawei" to mapOf(
        "push" to ":flavors:pushes-stub",
        "review" to ":flavors:review-stub",
        "walletConnect" to true,
        "firebase" to true,
        "googleServices" to true,
        "dependenciesInfo" to true,
        "updateUrl" to "https://appgallery.huawei.com/app/C109713129",
        "updateUrlEnv" to null,
        "abis" to listOf("armeabi-v7a", "arm64-v8a"),
        "isDefault" to false,
    ),
    "solana" to mapOf(
        "push" to ":flavors:fcm",
        "review" to ":flavors:review-stub",
        "walletConnect" to true,
        "firebase" to true,
        "googleServices" to true,
        "dependenciesInfo" to true,
        "updateUrl" to "solanadappstore://details?id=com.gemwallet.android",
        "updateUrlEnv" to null,
        "abis" to listOf("arm64-v8a"),
        "isDefault" to false,
    ),
    "samsung" to mapOf(
        "push" to ":flavors:fcm",
        "review" to ":flavors:review-stub",
        "walletConnect" to true,
        "firebase" to true,
        "googleServices" to true,
        "dependenciesInfo" to true,
        "updateUrl" to "https://apps.samsung.com/appquery/appDetail.as?appId=com.gemwallet.android",
        "updateUrlEnv" to null,
        "abis" to listOf("armeabi-v7a", "arm64-v8a"),
        "isDefault" to false,
    ),
    "emerald" to mapOf(
        "push" to ":flavors:fcm",
        "review" to ":flavors:review-stub",
        "walletConnect" to true,
        "firebase" to true,
        "googleServices" to true,
        "dependenciesInfo" to true,
        "updateUrl" to "https://apk.gemwallet.com/gem_wallet_latest.apk",
        "updateUrlEnv" to "UPDATE_URL",
        "abis" to listOf("armeabi-v7a", "arm64-v8a"),
        "isDefault" to false,
    ),
    "fdroid" to mapOf(
        "push" to ":flavors:pushes-stub",
        "review" to ":flavors:review-stub",
        "walletConnect" to false,
        "firebase" to false,
        "googleServices" to false,
        "dependenciesInfo" to false,
        "updateUrl" to "",
        "updateUrlEnv" to null,
        "abis" to listOf("armeabi-v7a", "arm64-v8a"),
        "isDefault" to false,
    ),
)

fun selectChannel(): String {
    val property = gradle.startParameter.projectProperties["channel"]?.takeIf { it.isNotBlank() }
    if (property != null) {
        require(CHANNELS.containsKey(property)) {
            "Unknown -Pchannel='$property'. Known channels: ${CHANNELS.keys}"
        }
        return property
    }
    val fromTask = gradle.startParameter.taskNames.firstNotNullOfOrNull { task ->
        val lowered = task.lowercase()
        CHANNELS.keys.firstOrNull { lowered.contains(it.lowercase()) }
    }
    if (fromTask != null) return fromTask
    return "google"
}

val activeChannel = selectChannel()
val activeFeatures = CHANNELS.getValue(activeChannel)

extra["gemChannels"] = CHANNELS
extra["gemChannel"] = activeChannel
extra["firebaseEnabled"] = activeFeatures["firebase"] == true
extra["walletConnectEnabled"] = activeFeatures["walletConnect"] == true
