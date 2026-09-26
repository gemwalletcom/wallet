package com.gemwallet.android.features.contacts.viewmodels.models

import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.wallet.core.primitives.ContactData

data class ContactRowUIModel(val contact: ContactData, val model: ListItemModel)
