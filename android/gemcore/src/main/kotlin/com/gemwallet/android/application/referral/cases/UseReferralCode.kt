package com.gemwallet.android.application.referral.cases

import com.wallet.core.primitives.Wallet

interface UseReferralCode {
    suspend fun useReferralCode(code: String, wallet: Wallet): Boolean
}
