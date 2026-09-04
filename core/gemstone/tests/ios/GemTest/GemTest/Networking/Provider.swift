// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone

public actor NativeProvider {
    let nodeConfig: [String: URL]
    let session: URLSession

    init(session: URLSession = .shared) {
        self.nodeConfig = [
            "ethereum": URL(string: "https://ethereum.publicnode.com")!,
            "optimism": URL(string: "https://mainnet.optimism.io")!,
            "thorchain": URL(string: "https://thornode.ninerealms.com")!,
            "solana": URL(string: "https://solana-rpc.publicnode.com")!,
            "smartchain": URL(string: "https://bsc-dataseed.bnbchain.org")!,
            "arbitrum": URL(string: "https://arb1.arbitrum.io/rpc")!,
            "base": URL(string: "https://mainnet.base.org")!,
            "polygon": URL(string: "https://polygon.drpc.org")!,
            "sui": URL(string: "https://fullnode.mainnet.sui.io")!,
            "abstract": URL(string: "https://api.mainnet.abs.xyz")!,
            "unichain": URL(string: "https://mainnet.unichain.org")!,
            "ink": URL(string: "https://rpc-qnd.inkonchain.com")!
        ]
        self.session = session
    }
}

extension NativeProvider: AlienProvider {
    public nonisolated func getEndpoint(chain: String) throws -> String {
        guard let url = nodeConfig[chain] else {
            throw AlienError.RequestError(msg: "\(chain) is not supported.")
        }
        return url.absoluteString
    }

    public func request(target: Gemstone.AlienTarget) async throws -> Gemstone.AlienResponse {
        print("==> handle request: \(target)")

        let (data, response) = try await self.session.data(for: target.asRequest())
        let status = (response as? HTTPURLResponse)?.statusCode

        print("<== response size: \(data.count)")

        return Gemstone.AlienResponse(status: status.map(UInt16.init), data: data)
    }
}
