package com.gemwallet.android.testkit

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NFTAssetId

fun mockNftAssetId() = NFTAssetId(chain = Chain.Ethereum, contractAddress = "0xasset", tokenId = "1")
