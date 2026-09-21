package com.gemwallet.android.features.settings.settings.viewmodels.localization

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.Appearance

@StringRes
fun Appearance.stringRes(): Int = when (this) {
    Appearance.System -> R.string.settings_appearance_system
    Appearance.Light -> R.string.settings_appearance_light
    Appearance.Dark -> R.string.settings_appearance_dark
}
