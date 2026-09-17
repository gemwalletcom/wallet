package com.gemwallet.android.features.settings.aboutus.presents.localization

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemAboutRow

@StringRes
internal fun GemAboutRow.stringRes(): Int = when (this) {
    GemAboutRow.TERMS_OF_SERVICE -> R.string.settings_terms_of_services
    GemAboutRow.PRIVACY_POLICY -> R.string.settings_privacy_policy
    GemAboutRow.WEBSITE -> R.string.settings_website
    GemAboutRow.COMMUNITY -> R.string.settings_community
    GemAboutRow.VERSION -> R.string.settings_version
}
