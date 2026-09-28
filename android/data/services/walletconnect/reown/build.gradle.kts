plugins {
    alias(libs.plugins.android.library)
    id("com.google.dagger.hilt.android")
    id("com.google.devtools.ksp")
}

android {
    namespace = "com.gemwallet.android.data.service.walletconnect.reown"
}

dependencies {
    implementation(project(":gemcore"))

    implementation(platform(libs.walletconnect.bom))
    implementation(libs.walletconnect.core) {
        exclude(group = "com.jakewharton.timber", module = "timber")
        exclude(group = "junit", module = "junit")
    }
    implementation(libs.walletconnect.web3wallet) {
        exclude(group = "junit", module = "junit")
    }

    implementation(libs.hilt.android)
    ksp(libs.hilt.compiler)
}
