package com.gemwallet.android.features.create_wallet.viewmodels.localization

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemSecurityReminderItem

@StringRes
internal fun GemSecurityReminderItem.titleRes(): Int = when (this) {
    GemSecurityReminderItem.KEEP_SAFE -> R.string.onboarding_security_create_wallet_keep_safe_title
    GemSecurityReminderItem.DO_NOT_SHARE -> R.string.onboarding_security_create_wallet_do_not_share_title
    GemSecurityReminderItem.NO_RECOVERY -> R.string.onboarding_security_create_wallet_no_recovery_title
}

@StringRes
internal fun GemSecurityReminderItem.subtitleRes(): Int = when (this) {
    GemSecurityReminderItem.KEEP_SAFE -> R.string.onboarding_security_create_wallet_keep_safe_subtitle
    GemSecurityReminderItem.DO_NOT_SHARE -> R.string.onboarding_security_create_wallet_do_not_share_subtitle
    GemSecurityReminderItem.NO_RECOVERY -> R.string.onboarding_security_create_wallet_no_recovery_subtitle
}
