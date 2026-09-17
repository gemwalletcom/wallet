package com.gemwallet.android.features.settings.aboutus.presents.models

import android.content.Context
import com.gemwallet.android.AppUrl
import com.gemwallet.android.features.settings.aboutus.presents.localization.stringRes
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.property.icon
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.models.ListSection
import uniffi.gemstone.GemAboutRow
import uniffi.gemstone.PublicUrl
import uniffi.gemstone.aboutSections
import uniffi.gemstone.communityLinks

data class AboutRowUIModel(
    val url: String?,
    val model: ListItemModel,
)

internal fun aboutSections(context: Context, version: String): List<ListSection<AboutRowUIModel>> =
    aboutSections().mapIndexed { index, section ->
        if (section.rows == listOf(GemAboutRow.COMMUNITY)) {
            ListSection(
                id = index.toString(),
                title = context.getString(GemAboutRow.COMMUNITY.stringRes()),
                items = communityLinks().map { link ->
                    AboutRowUIModel(url = link.url, model = ListItemModel(title = context.getString(link.linkType.stringRes()), image = ListItemImage.Drawable(link.linkType.icon)))
                },
            )
        } else {
            ListSection(id = index.toString(), items = section.rows.map { it.uiModel(context, version) })
        }
    }

private fun GemAboutRow.uiModel(context: Context, version: String): AboutRowUIModel = when (this) {
    GemAboutRow.TERMS_OF_SERVICE -> AboutRowUIModel(AppUrl.page(PublicUrl.TERMS_OF_SERVICE), ListItemModel(title = context.getString(stringRes())))
    GemAboutRow.PRIVACY_POLICY -> AboutRowUIModel(AppUrl.page(PublicUrl.PRIVACY_POLICY), ListItemModel(title = context.getString(stringRes())))
    GemAboutRow.WEBSITE -> AboutRowUIModel(AppUrl.page(PublicUrl.WEBSITE), ListItemModel(title = context.getString(stringRes())))
    GemAboutRow.COMMUNITY -> AboutRowUIModel(null, ListItemModel(title = context.getString(stringRes())))
    GemAboutRow.VERSION -> AboutRowUIModel(null, ListItemModel(title = context.getString(stringRes()), subtitle = version))
}
