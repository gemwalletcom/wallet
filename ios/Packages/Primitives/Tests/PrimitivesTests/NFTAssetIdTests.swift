// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Testing

struct NFTAssetIdTests {
    @Test
    func fromIdAndBackToId() throws {
        let asset = try NFTAssetId.from(id: "ethereum_0xabc::1")
        #expect(asset == NFTAssetId(chain: .ethereum, contractAddress: "0xabc", tokenId: "1"))
        #expect(asset.identifier == "ethereum_0xabc::1")

        let collection = try NFTCollectionId.from(id: "ethereum_0xabc")
        #expect(collection == NFTCollectionId(chain: .ethereum, contractAddress: "0xabc"))
        #expect(collection.identifier == "ethereum_0xabc")

        let tonAsset = try NFTAssetId.from(id: "ton_EQabc::EQtoken::1")
        #expect(tonAsset.chain == .ton)
        #expect(tonAsset.contractAddress == "EQabc")
        #expect(tonAsset.tokenId == "EQtoken::1")
        #expect(tonAsset.identifier == "ton_EQabc::EQtoken::1")

        let tonUnderscoreAsset = try NFTAssetId.from(id: "ton_EQ_addr_with_underscores::EQ_token_with_underscores")
        #expect(tonUnderscoreAsset.chain == .ton)
        #expect(tonUnderscoreAsset.contractAddress == "EQ_addr_with_underscores")
        #expect(tonUnderscoreAsset.tokenId == "EQ_token_with_underscores")
        #expect(tonUnderscoreAsset.identifier == "ton_EQ_addr_with_underscores::EQ_token_with_underscores")

        let tonUnderscoreCollection = try NFTCollectionId.from(id: "ton_EQ_addr_with_underscores")
        #expect(tonUnderscoreCollection.chain == .ton)
        #expect(tonUnderscoreCollection.contractAddress == "EQ_addr_with_underscores")

        #expect(throws: Error.self) { try NFTAssetId.from(id: "ethereum_0xabc") }
        #expect(throws: Error.self) { try NFTAssetId.from(id: "unknownchain_0xabc::1") }
        #expect(throws: Error.self) { try NFTAssetId.from(id: "nounderscore") }
        #expect(throws: Error.self) { try NFTCollectionId.from(id: "nounderscore") }
        #expect(throws: Error.self) { try NFTCollectionId.from(id: "unknownchain_0xabc") }

        let encoded = try JSONEncoder().encode(asset)
        #expect(String(data: encoded, encoding: .utf8) == "\"ethereum_0xabc::1\"")
        #expect(try JSONDecoder().decode(NFTAssetId.self, from: encoded) == asset)
    }
}
