package com.gemwallet.android.features.onboarding.localization

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemAcceptTermsItem

@StringRes
internal fun GemAcceptTermsItem.stringRes(): Int = when (this) {
    GemAcceptTermsItem.SELF_CUSTODY -> R.string.onboarding_accept_terms_item1_message
    GemAcceptTermsItem.RECOVERY -> R.string.onboarding_accept_terms_item2_message
    GemAcceptTermsItem.RESPONSIBILITY -> R.string.onboarding_accept_terms_item3_message
}
