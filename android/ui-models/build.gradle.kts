plugins {
    alias(libs.plugins.android.library)
}

android {
    namespace = "com.gemwallet.android.ui.models"

    packaging {
        resources {
            excludes += "/META-INF/LICENSE.md"
            excludes += "/META-INF/LICENSE-notice.md"
            excludes += "META-INF/versions/9/OSGI-INF/MANIFEST.MF"
        }
    }
}

dependencies {
    api(project(":gemcore"))
    api(libs.lifecycle.viewmodel.savedstate)
    testImplementation(testFixtures(project(":gemcore")))
    testImplementation(libs.junit)
    testImplementation(libs.kotlinx.coroutines.test)
}
