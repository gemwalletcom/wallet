package com.gemwallet.android.testkit

import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.NFTAssetData
import com.wallet.core.primitives.NFTCollection

fun mockNftAssetData(collection: NFTCollection = mockNftCollection(), asset: NFTAsset = mockNftAsset(collectionId = collection.id)) = NFTAssetData(collection = collection, asset = asset)
