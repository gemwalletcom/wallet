package com.gemwallet.android.features.settings.security.viewmodels.models

import android.content.Context
import androidx.annotation.StringRes
import com.gemwallet.android.features.settings.security.viewmodels.localization.stringRes
import com.gemwallet.android.ui.components.list_item.ListItemModel
import uniffi.gemstone.GemSecurityRow
import uniffi.gemstone.lockPeriodFromMinutes

sealed interface SecurityRowUIModel {
    data class Authentication(val model: ListItemModel, val isEnabled: Boolean) : SecurityRowUIModel
    data class LockPeriod(val model: ListItemModel, val options: List<LockPeriodOption>) : SecurityRowUIModel
    data class HideBalance(val model: ListItemModel, val isEnabled: Boolean) : SecurityRowUIModel
}

data class LockPeriodOption(val minutes: Int, @StringRes val title: Int, val isSelected: Boolean = false)

internal fun GemSecurityRow.uiModel(
    context: Context,
    authRequired: Boolean,
    lockInterval: Int,
    hideBalances: Boolean,
    lockPeriods: List<LockPeriodOption>,
): SecurityRowUIModel? = when (this) {
    GemSecurityRow.AUTHENTICATION -> SecurityRowUIModel.Authentication(ListItemModel(title = context.getString(stringRes())), authRequired)
    GemSecurityRow.LOCK_PERIOD -> SecurityRowUIModel.LockPeriod(
        model = ListItemModel(
            title = context.getString(stringRes()),
            subtitle = context.getString(lockPeriodFromMinutes(lockInterval.toUInt()).stringRes()),
        ),
        options = lockPeriods.map { it.copy(isSelected = it.minutes == lockInterval) },
    )
    GemSecurityRow.PRIVACY_LOCK -> null
    GemSecurityRow.HIDE_BALANCE -> SecurityRowUIModel.HideBalance(ListItemModel(title = context.getString(stringRes())), hideBalances)
}
