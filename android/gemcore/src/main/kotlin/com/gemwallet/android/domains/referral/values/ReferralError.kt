package com.gemwallet.android.domains.referral.values

sealed class ReferralError(message: String = "") : Exception(message) {
    object InsufficientPoints : ReferralError()
}
