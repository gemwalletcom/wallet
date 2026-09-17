package com.gemwallet.android.features.settings.security.viewmodels.localization

import uniffi.gemstone.GemSecurityRow
import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemLockPeriod

@StringRes
internal fun GemLockPeriod.stringRes(): Int = when (this) {
    GemLockPeriod.IMMEDIATE -> R.string.lock_immediately
    GemLockPeriod.ONE_MINUTE -> R.string.lock_one_minute
    GemLockPeriod.FIVE_MINUTES -> R.string.lock_five_minutes
    GemLockPeriod.FIFTEEN_MINUTES -> R.string.lock_fifteen_minutes
    GemLockPeriod.ONE_HOUR -> R.string.lock_one_hour
    GemLockPeriod.SIX_HOURS -> R.string.lock_six_hours
}

@StringRes
fun GemSecurityRow.stringRes(): Int = when (this) {
    GemSecurityRow.AUTHENTICATION -> R.string.settings_enable_passcode
    GemSecurityRow.LOCK_PERIOD -> R.string.lock_require_authentication
    GemSecurityRow.PRIVACY_LOCK -> R.string.lock_privacy_lock
    GemSecurityRow.HIDE_BALANCE -> R.string.settings_hide_balance
}
