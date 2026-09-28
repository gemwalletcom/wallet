plugins {
    alias(libs.plugins.android.library)
    id("com.google.dagger.hilt.android")
    id("com.google.devtools.ksp")
}

android {
    namespace = "com.gemwallet.android.flovers"
}

dependencies {
    implementation(project(":gemcore"))

    implementation(libs.firebase.messaging)

    implementation(libs.hilt.android)
    ksp(libs.hilt.compiler)
}
