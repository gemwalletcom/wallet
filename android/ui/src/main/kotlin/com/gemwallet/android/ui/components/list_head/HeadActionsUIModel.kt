package com.gemwallet.android.ui.components.list_head

import androidx.annotation.DrawableRes
import androidx.annotation.StringRes
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.style.iconRes
import uniffi.gemstone.GemHeaderActions
import uniffi.gemstone.GemHeaderButtonKind

data class HeadActionUIModel(@StringRes val title: Int, @DrawableRes val icon: Int, val enabled: Boolean, val onClick: () -> Unit, val testTag: String? = null)

sealed interface HeadActionsUIModel {
    data object WatchOnly : HeadActionsUIModel
    data class Buttons(val items: List<HeadActionUIModel>) : HeadActionsUIModel
}

fun GemHeaderActions.uiModel(
    onTransfer: (() -> Unit)?,
    onReceive: (() -> Unit)?,
    onBuy: (() -> Unit)?,
    onSwap: (() -> Unit)?,
    onDeposit: (() -> Unit)? = null,
    onWithdraw: (() -> Unit)? = null,
    onMore: (() -> Unit)? = null,
): HeadActionsUIModel = when (this) {
    GemHeaderActions.WatchOnly -> HeadActionsUIModel.WatchOnly

    is GemHeaderActions.Buttons -> HeadActionsUIModel.Buttons(
        buttons.mapNotNull { button ->
            val onClick = when (button.kind) {
                GemHeaderButtonKind.SEND -> onTransfer
                GemHeaderButtonKind.RECEIVE -> onReceive
                GemHeaderButtonKind.BUY -> onBuy
                GemHeaderButtonKind.SWAP -> onSwap
                GemHeaderButtonKind.DEPOSIT -> onDeposit
                GemHeaderButtonKind.WITHDRAW -> onWithdraw
                GemHeaderButtonKind.MORE -> onMore
            } ?: return@mapNotNull null
            HeadActionUIModel(
                title = button.kind.stringRes(),
                icon = button.kind.iconRes(),
                enabled = button.isEnabled,
                onClick = onClick,
                testTag = when (button.kind) {
                    GemHeaderButtonKind.BUY -> "assetBuy"

                    GemHeaderButtonKind.SEND, GemHeaderButtonKind.RECEIVE, GemHeaderButtonKind.SWAP,
                    GemHeaderButtonKind.DEPOSIT, GemHeaderButtonKind.WITHDRAW, GemHeaderButtonKind.MORE,
                    -> null
                },
            )
        },
    )
}
