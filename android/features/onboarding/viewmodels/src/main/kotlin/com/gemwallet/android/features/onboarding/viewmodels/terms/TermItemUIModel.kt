package com.gemwallet.android.features.onboarding.viewmodels.terms

import androidx.annotation.StringRes
import com.gemwallet.android.ext.GemConstants
import com.gemwallet.android.ui.localization.stringRes

data class TermItemUIModel(@param:StringRes val description: Int)

fun termItems(): List<TermItemUIModel> = GemConstants.acceptTermsItems.map { TermItemUIModel(it.stringRes()) }
