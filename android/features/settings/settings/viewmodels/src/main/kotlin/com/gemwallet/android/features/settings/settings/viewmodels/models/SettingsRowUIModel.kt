package com.gemwallet.android.features.settings.settings.viewmodels.models

import android.content.Context
import com.gemwallet.android.features.settings.settings.viewmodels.localization.stringRes
import com.gemwallet.android.features.settings.settings.viewmodels.style.icon
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.models.actions.SettingsSceneAction
import uniffi.gemstone.GemSettingsRow

data class SettingsRowUIModel(
    val action: SettingsSceneAction,
    val model: ListItemModel,
    val opensDeveloperMenu: Boolean = false,
)

internal fun GemSettingsRow.uiModel(context: Context, walletsCount: Int): SettingsRowUIModel = when (this) {
    GemSettingsRow.WALLETS -> SettingsRowUIModel(SettingsSceneAction.Wallets, listItem(context, subtitle = walletsCount.toString()))
    GemSettingsRow.SECURITY -> SettingsRowUIModel(SettingsSceneAction.Security, listItem(context))
    GemSettingsRow.NOTIFICATIONS -> SettingsRowUIModel(SettingsSceneAction.Notifications, listItem(context))
    GemSettingsRow.PREFERENCES -> SettingsRowUIModel(SettingsSceneAction.Preferences, listItem(context))
    GemSettingsRow.WALLET_CONNECT -> SettingsRowUIModel(SettingsSceneAction.Bridges, listItem(context))
    GemSettingsRow.SUPPORT -> SettingsRowUIModel(SettingsSceneAction.Support, listItem(context))
    GemSettingsRow.REWARDS -> SettingsRowUIModel(SettingsSceneAction.Referral, listItem(context))
    GemSettingsRow.ABOUT_US -> SettingsRowUIModel(SettingsSceneAction.AboutUs, listItem(context), opensDeveloperMenu = true)
    GemSettingsRow.DEVELOPER -> SettingsRowUIModel(SettingsSceneAction.Develop, listItem(context))
}

private fun GemSettingsRow.listItem(context: Context, subtitle: String? = null): ListItemModel = ListItemModel(
    title = context.getString(stringRes()),
    subtitle = subtitle,
    image = ListItemImage.Drawable(icon()),
)
