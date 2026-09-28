plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.compose.compiler)
    id("com.google.devtools.ksp")
}

android {
    namespace = "com.gemwallet.android.features.swap.presents"

    buildFeatures {
        compose = true
    }
}

dependencies {
    implementation(project(":ui"))
    implementation(project(":features:swap:viewmodels"))
    implementation(project(":features:assets:presents"))
    implementation(project(":features:assets:viewmodels"))

    implementation(libs.hilt.lifecycle.viewmodel.compose)
}
