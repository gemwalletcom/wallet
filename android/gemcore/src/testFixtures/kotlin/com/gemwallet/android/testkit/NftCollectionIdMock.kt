package com.gemwallet.android.testkit

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NFTCollectionId

fun mockNftCollectionId(contractAddress: String = "0xcollection") = NFTCollectionId(chain = Chain.Ethereum, contractAddress = contractAddress)
