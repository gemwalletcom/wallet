plugins {
    alias(libs.plugins.android.library)
}

android {
    namespace = "com.gemwallet.android.data.services.nativeprovider"
}

dependencies {
    implementation(project(":gemcore"))
    implementation(libs.okhttp)

    implementation(libs.kotlinx.coroutines.android)

    testImplementation(libs.junit)
    testImplementation(libs.mockk.android)
}
