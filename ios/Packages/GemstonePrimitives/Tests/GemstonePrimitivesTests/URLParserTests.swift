// Copyright (c). Gem Wallet. All rights reserved.

@testable import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing

struct URLParserTests {
    @Test
    func deeplink() throws {
        #expect(try URLParser.from(url: #require(DeepLinkMock.assetBitcoin.asURL)) == .deeplink(.asset(.mock())))
        #expect(try URLParser.from(url: #require(DeepLinkMock.assetEthereumToken.asURL)) == .deeplink(.asset(.mockEthereumUSDT())))
        #expect(try URLParser.from(url: #require(DeepLinkMock.perpetualsGem.asURL)) == .deeplink(.perpetuals))
        #expect(try URLParser.from(url: #require(DeepLinkMock.rewards.asURL)) == .deeplink(.rewards(code: "gemcoder")))
        #expect(try URLParser.from(url: #require(DeepLinkMock.giftGem.asURL)) == .deeplink(.gift(code: nil)))
    }

    @Test
    func walletConnect() throws {
        #expect(try URLParser.from(url: #require(DeepLinkMock.walletConnectConnect.asURL)) == .walletConnect(.connect(uri: "wc:topic@2")))
        #expect(try URLParser.from(url: #require(DeepLinkMock.walletConnectRequest.asURL)) == .walletConnect(.request))
        #expect(try URLParser.from(url: #require(DeepLinkMock.walletConnectSession.asURL)) == .walletConnect(.session("abc123")))
    }

    @Test
    func invalidURL() throws {
        #expect(throws: URLParserError.self) { try URLParser.from(url: #require(DeepLinkMock.noPath.asURL)) }
        #expect(throws: URLParserError.self) { try URLParser.from(url: #require(DeepLinkMock.badPath.asURL)) }
        #expect(throws: URLParserError.self) { try URLParser.from(url: #require(DeepLinkMock.badHost.asURL)) }
    }
}
