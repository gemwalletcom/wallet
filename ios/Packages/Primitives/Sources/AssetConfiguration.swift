import Foundation

public struct AssetConfiguration: Sendable {
    public static let supportedChainsWithTokens: [Chain] = [
        [
            .solana,
            .ton,
            .sui,
            .aptos,
            .tron,
            .algorand,
            .xrp,
            .stellar,
            .hyperCore,
        ],
        EVMChain.allCases.compactMap { Chain(rawValue: $0.rawValue) },
    ]
    .flatMap(\.self)

    public static let allChains: [Chain] = Chain.allCases
}
