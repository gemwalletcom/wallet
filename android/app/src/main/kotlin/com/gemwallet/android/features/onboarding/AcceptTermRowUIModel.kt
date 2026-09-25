package com.gemwallet.android.features.onboarding

import androidx.annotation.StringRes
import com.gemwallet.android.ext.GemConstants
import com.gemwallet.android.ui.localization.stringRes

data class AcceptTermRowUIModel(@param:StringRes val description: Int)

fun acceptTermRows(): List<AcceptTermRowUIModel> = GemConstants.acceptTermsItems.map { AcceptTermRowUIModel(it.stringRes()) }
