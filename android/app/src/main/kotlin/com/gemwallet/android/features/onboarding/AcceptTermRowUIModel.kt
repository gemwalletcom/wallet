package com.gemwallet.android.features.onboarding

import androidx.annotation.StringRes
import com.gemwallet.android.features.onboarding.localization.stringRes
import uniffi.gemstone.acceptTermsItems

data class AcceptTermRowUIModel(
    @param:StringRes val description: Int,
)

fun acceptTermRows(): List<AcceptTermRowUIModel> = acceptTermsItems().map { AcceptTermRowUIModel(it.stringRes()) }
