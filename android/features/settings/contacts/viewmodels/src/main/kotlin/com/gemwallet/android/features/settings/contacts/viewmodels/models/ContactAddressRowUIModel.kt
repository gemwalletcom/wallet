package com.gemwallet.android.features.settings.contacts.viewmodels.models

import android.content.Context
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemSymbol
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ContactAddress
import uniffi.gemstone.GemAddressFormatStyle
import uniffi.gemstone.GemContactEditorServiceInterface

data class ContactAddressRowUIModel(val address: ContactAddress, val model: ListItemModel)

internal fun List<ContactAddress>.rows(service: GemContactEditorServiceInterface): List<ContactAddressRowUIModel> = map { address ->
    ContactAddressRowUIModel(
        address = address,
        model = ListItemModel(
            title = address.chain.networkName(),
            titleExtra = service.formatAddress(address.address, address.chain.string, GemAddressFormatStyle.Short),
            image = ListItemImage.Asset(AssetId(address.chain)),
        ),
    )
}

internal fun addAddressListItem(context: Context): ListItemModel = ListItemModel(
    title = context.getString(R.string.common_address),
    titleStyle = ListItemTextStyle.Primary,
    image = ListItemImage.Symbol(ListItemSymbol.Add, tint = ListItemTextStyle.Primary),
)
