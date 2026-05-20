// Copyright (c). Gem Wallet. All rights reserved.

enum DeepLinkMock {
    static let assetBitcoin = "https://gemwallet.com/tokens/bitcoin"
    static let assetBitcoinGem = "gem://tokens/bitcoin"
    static let assetEthereumToken = "https://gemwallet.com/tokens/ethereum/0xdAC17F958D2ee523a2206206994597C13D831ec7"
    static let swap = "https://gemwallet.com/swap/ethereum/ethereum_0xdAC17F958D2ee523a2206206994597C13D831ec7"
    static let perpetuals = "https://gemwallet.com/perpetuals"
    static let perpetualsGem = "gem://perpetuals"
    static let rewards = "https://gemwallet.com/rewards?code=gemcoder"
    static let gift = "https://gemwallet.com/gift?code=giftcode123"
    static let giftGem = "gem://gift"
    static let buy = "https://gemwallet.com/buy/bitcoin?amount=100"
    static let sell = "https://gemwallet.com/sell/ethereum"
    static let setPriceAlert = "https://gemwallet.com/setPriceAlert/bitcoin?price=2.5"
    static let walletConnectConnect = "gem://wc?uri=wc:topic@2"
    static let walletConnectRequest = "gem://wc?requestId=1"
    static let walletConnectSession = "gem://wc?sessionTopic=abc123"
    static let noPath = "gem://"
    static let badPath = "gem://invalidpath"
    static let badHost = "https://example.com/tokens/bitcoin"
}
