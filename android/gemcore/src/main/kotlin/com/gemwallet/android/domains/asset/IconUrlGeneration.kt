package com.gemwallet.android.domains.asset

import com.gemwallet.android.ext.chainConfig
import com.gemwallet.android.ext.toChain
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemImage

fun Chain.iconChain(): Chain = chainConfig().iconChain.toChain()

fun getListIconUrl(listId: String): String = GemImage.AssetList(listId).url()
