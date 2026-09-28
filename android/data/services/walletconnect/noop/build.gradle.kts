plugins {
    alias(libs.plugins.android.library)
    id("com.google.dagger.hilt.android")
    id("com.google.devtools.ksp")
}

android {
    namespace = "com.gemwallet.android.data.service.walletconnect.noop"
}

dependencies {
    implementation(project(":gemcore"))

    implementation(libs.hilt.android)
    ksp(libs.hilt.compiler)
}
