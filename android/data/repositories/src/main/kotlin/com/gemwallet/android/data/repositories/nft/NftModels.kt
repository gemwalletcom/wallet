package com.gemwallet.android.data.repositories.nft

import com.gemwallet.android.data.service.store.database.entities.DbNFTAsset
import com.gemwallet.android.data.service.store.database.entities.DbNFTCollection
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.NFTCollection
import com.wallet.core.primitives.NFTData
import com.wallet.core.primitives.NFTImages
import com.wallet.core.primitives.NFTResource
import uniffi.gemstone.GemNftRulesService

fun DbNFTAsset.toNftData(collection: DbNFTCollection) = NFTData(
    collection = collection.toCollectionModel(),
    assets = listOf(toAssetModel()),
)

fun List<DbNFTCollection>.toCollectionModels() = map { it.toCollectionModel() }

internal fun DbNFTCollection.toCollectionModel(nftRules: GemNftRulesService = GemNftRulesService()) = NFTCollection(
    id = id,
    name = name,
    description = description,
    chain = chain,
    contractAddress = contractAddress,
    images = NFTImages(NFTResource(imageUrl, "")),
    status = nftRules.collectionStatus(status?.toJson()).decodeJson(),
    links = links ?: emptyList(),
)

fun List<DbNFTAsset>.toAssetModels(): List<NFTAsset> = map { it.toAssetModel() }

internal fun DbNFTAsset.toAssetModel() = NFTAsset(
    id = id,
    collectionId = collectionId,
    tokenId = tokenId,
    tokenType = tokenType,
    contractAddress = contractAddress,
    name = name,
    description = description,
    chain = chain,
    resource = NFTResource("", ""),
    images = NFTImages(NFTResource(imageUrl, "")),
    attributes = attributes ?: emptyList(),
)
