import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.jetbrains.kotlin.gradle.tasks.KotlinJvmCompile

plugins {
    id("com.android.application")
    alias(libs.plugins.compose.compiler)
    id("com.google.dagger.hilt.android")
    id("kotlinx-serialization")
    id("com.google.devtools.ksp")
    id("androidx.room")
}

apply(from = "$rootDir/gradle/channels.gradle.kts")

@Suppress("UNCHECKED_CAST")
val gemChannels: Map<String, Map<String, Any?>> by extra
val gemChannel: String by extra

gradle.startParameter.projectProperties["channel"]?.takeIf { it.isNotBlank() }?.let { explicit ->
    val requestedChannels = gradle.startParameter.taskNames
        .asSequence()
        .flatMap { task ->
            gemChannels.keys.asSequence().filter { channel ->
                task.contains("assemble$channel", ignoreCase = true) || task.contains("bundle$channel", ignoreCase = true)
            }
        }
        .toSet()
        .filter { it != explicit }
    require(requestedChannels.isEmpty()) {
        "Channel mismatch: -Pchannel=$explicit but requested task(s) for $requestedChannels. Build one channel per invocation."
    }
}

if (file("google-services.json").isFile) {
    apply(plugin = "com.google.gms.google-services")

    tasks.configureEach {
        if (name.startsWith("processFdroid") && name.endsWith("GoogleServices")) {
            enabled = false
        }
    }
}

repositories {
    google()
    mavenCentral()
}

android {
    namespace = "com.gemwallet.android"
    compileSdk = 37
    ndkVersion = libs.versions.androidNdk.get()

    val channelDimension by extra("channel")
    flavorDimensions.add(channelDimension)

    defaultConfig {
        applicationId = "com.gemwallet.android"
        minSdk = 28
        targetSdk = 37
        versionCode = 792
        versionName = "2.93"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        vectorDrawables {
            useSupportLibrary = true
        }

        splits {
            abi {
                isEnable = false
                include("arm64-v8a", "armeabi-v7a")
                isUniversalApk = false
            }
        }
    }
    productFlavors {
        gemChannels.forEach { (name, channel) ->
            create(name) {
                dimension = channelDimension
                isDefault = channel["isDefault"] == true
                ndk {
                    @Suppress("UNCHECKED_CAST")
                    abiFilters.addAll(channel.getValue("abis") as List<String>)
                }
                val updateUrl = channel.getValue("updateUrl") as String
                val updateUrlEnv = channel["updateUrlEnv"] as String?
                val resolvedUpdateUrl = (updateUrlEnv?.let { providers.environmentVariable(it).orNull } ?: updateUrl)
                    .removeSurrounding("\"")
                    .replace("\\", "\\\\")
                    .replace("\"", "\\\"")
                buildConfigField("String", "UPDATE_URL", "\"$resolvedUpdateUrl\"")
            }
        }
    }
    signingConfigs {
        create("release") {
            keyAlias = System.getenv("ANDROID_KEYSTORE_ALIAS")
            keyPassword = System.getenv("ANDROID_KEYSTORE_PASSWORD")
            storeFile = file(System.getenv("ANDROID_KEYSTORE_FILENAME") ?: "release.keystore")
            storePassword = System.getenv("ANDROID_KEYSTORE_PASSWORD")
            enableV1Signing = false
            enableV2Signing = true
        }
    }

    buildTypes {
        getByName("debug") {
            isMinifyEnabled = false
            isShrinkResources = false
            isDebuggable = true

            if (System.getenv("UNIT_TESTS") == "true") {
                ndk {
                    abiFilters.remove("arm64-v8a")
                    abiFilters.remove("armeabi-v7a")
                    abiFilters.add("x86_64")
                }

                splits {
                    abi {
                        reset()
                        isEnable = false
                        include("x86_64")
                        isUniversalApk = false
                    }
                }
            }

            buildConfigField("String", "TEST_PHRASE", "${System.getenv("TEST_PHRASE")}")
        }

        getByName("release") {
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro",
            )
            isMinifyEnabled = true
            isShrinkResources = true
            isDebuggable = false
            if (System.getenv("SKIP_SIGN") == "true") {
                signingConfig = null
            } else {
                signingConfig = signingConfigs.getByName("release")
            }
        }

    }

    packaging {
        resources {
            excludes += "META-INF/*"
            excludes += "META-INF/DEPENDENCIES"
            excludes += "/META-INF/LICENSE-notice.md"
            excludes += "/META-INF/LICENSE.md"
            excludes += "META-INF/versions/9/OSGI-INF/MANIFEST.MF"
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    tasks.withType<KotlinJvmCompile>().configureEach {
        compilerOptions {
            jvmTarget.set(JvmTarget.JVM_17)
            freeCompilerArgs.add("-opt-in=kotlin.RequiresOptIn")
        }
    }
    buildFeatures {
        compose = true
        buildConfig = true
    }
    dependenciesInfo {
        val enabled = gemChannel != "fdroid"
        includeInApk = enabled
        includeInBundle = enabled
    }
    packaging {
        jniLibs {
            useLegacyPackaging = true
        }
    }


    androidResources {
        generateLocaleConfig = true
    }

    room {
        schemaDirectory("$projectDir/schemas")
    }

    lint {
        disable += "Instantiatable"
        checkGeneratedSources = true
        checkDependencies = false
    }
}

dependencies {
    implementation(project(":blockchain"))
    implementation(project(":ui"))
    implementation(project(":data:repositories"))
    implementation(project(":data:coordinators"))

    // Features
    implementation(project(":features:activities:presents"))
    implementation(project(":features:activities:viewmodels"))
    implementation(project(":features:add_asset:presents"))
    implementation(project(":features:add_asset:viewmodels"))
    implementation(project(":features:asset:presents"))
    implementation(project(":features:asset:viewmodels"))
    implementation(project(":features:asset_select:presents"))
    implementation(project(":features:asset_select:viewmodels"))
    implementation(project(":features:banner:presents"))
    implementation(project(":features:banner:viewmodels"))
    implementation(project(":features:buy:presents"))
    implementation(project(":features:buy:viewmodels"))
    implementation(project(":features:confirm:presents"))
    implementation(project(":features:confirm:viewmodels"))
    implementation(project(":features:transfer_amount:presents"))
    implementation(project(":features:transfer_amount:viewmodels"))
    implementation(project(":features:swap:presents"))
    implementation(project(":features:swap:viewmodels"))
    implementation(project(":features:receive:presents"))
    implementation(project(":features:receive:viewmodels"))
    implementation(project(":features:wallets:presents"))
    implementation(project(":features:wallets:viewmodels"))
    implementation(project(":features:import_wallet:presents"))
    implementation(project(":features:import_wallet:viewmodels"))
    implementation(project(":features:create_wallet:presents"))
    implementation(project(":features:create_wallet:viewmodels"))
    implementation(project(":features:earn:stake:presents"))
    implementation(project(":features:earn:stake:viewmodels"))
    implementation(project(":features:earn:delegation:presents"))
    implementation(project(":features:earn:delegation:viewmodels"))
    implementation(project(":features:settings:aboutus:presents"))
    implementation(project(":features:settings:currency:presents"))
    implementation(project(":features:settings:currency:viewmodels"))
    implementation(project(":features:settings:develop:presents"))
    implementation(project(":features:settings:develop:viewmodels"))
    implementation(project(":features:settings:in_app_notifications:presents"))
    implementation(project(":features:settings:in_app_notifications:viewmodels"))
    implementation(project(":features:settings:networks:presents"))
    implementation(project(":features:settings:networks:viewmodels"))
    implementation(project(":features:settings:price_alerts:presents"))
    implementation(project(":features:settings:price_alerts:viewmodels"))
    implementation(project(":features:settings:contacts:presents"))
    implementation(project(":features:settings:contacts:viewmodels"))
    implementation(project(":features:settings:security:presents"))
    implementation(project(":features:settings:security:viewmodels"))
    implementation(project(":features:settings:settings:presents"))
    implementation(project(":features:settings:settings:viewmodels"))
    implementation(project(":features:recipient:presents"))
    implementation(project(":features:recipient:viewmodels"))
    implementation(project(":features:nft:presents"))
    implementation(project(":features:wallet-details:presents"))
    implementation(project(":features:bridge:presents"))
    implementation(project(":features:bridge:viewmodels"))
    implementation(project(":features:assets:presents"))
    implementation(project(":features:assets:viewmodels"))
    implementation(project(":features:perpetual:presents"))
    implementation(project(":features:referral:viewmodels"))
    implementation(project(":features:referral:presents"))

    implementation(libs.ktx.core)
    implementation(libs.androidx.core.splashscreen)

    implementation(libs.tink)

    implementation(libs.hilt.lifecycle.viewmodel.compose)
    implementation(libs.hilt.android)
    ksp(libs.hilt.compiler)

    implementation(libs.lifecycle.process)
    implementation(libs.lifecycle.runtime.ktx)
    implementation(libs.lifecycle.runtime.compose)
    implementation(libs.lifecycle.viewmodel.savedstate)
    implementation(libs.datastore)

    implementation(libs.navigation3.runtime)
    implementation(libs.navigation3.ui)

    implementation(libs.widgets)
    implementation(libs.widgets.material3)

    // Legacy encrypted preferences migration
    implementation(libs.androidx.security.crypto)
    // Auth
    implementation(libs.androidx.biometric)

    gemChannels.forEach { (name, channel) ->
        val configuration = "${name}Implementation"
        listOf("push", "review", "walletConnect").forEach { key ->
            val module = channel[key] as String
            if (rootProject.findProject(module) != null) {
                add(configuration, project(module))
            }
        }
    }

    // Preview
    debugImplementation(libs.androidx.ui.tooling)
    implementation(libs.androidx.ui.tooling.preview)

    // Tests
    testImplementation(testFixtures(project(":gemcore")))
    testImplementation(libs.mockk.android)
    testImplementation(libs.junit)
    testImplementation(libs.androidx.junit)
    testImplementation(libs.androidx.junit.runner)
    testImplementation(libs.androidx.junit.ktx)
    testImplementation(libs.kotlinx.coroutines.test)
    androidTestImplementation(libs.turbine)
    androidTestImplementation(libs.mockk.android)
    androidTestImplementation(libs.mockk.agent)
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso.core)
    androidTestImplementation(libs.androidx.ui.test.junit4)
    androidTestImplementation(libs.androidx.uiautomator)
    androidTestImplementation(testFixtures(project(":gemcore")))
}
