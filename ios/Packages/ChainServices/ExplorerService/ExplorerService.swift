// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.Config
import class Gemstone.Explorer
import GemstonePrimitives
import Preferences
import Primitives

public struct ExplorerService {
    private let preferences: any ExplorerPreferencesStorable

    public static let standard: ExplorerService = .init()

    public init(preferences: any ExplorerPreferencesStorable = ExplorerPreferences()) {
        self.preferences = preferences
    }

    public static func explorers(chain: Chain) -> [String] {
        Gemstone.Config.shared.getBlockExplorers(chain: chain.id)
    }

    public func transactionUrl(chain: Chain, hash: String) -> BlockExplorerLink {
        let (name, explorer) = getExplorer(chain: chain)
        return makeLink(name: name, url: explorer.getTransactionUrl(explorerName: name, transactionId: hash))!
    }

    public func swapTransactionUrl(chain: Chain, provider: String, input: ExplorerInput) -> BlockExplorerLink? {
        let (name, explorer) = getExplorer(chain: chain)
        guard let url = explorer.getTransactionSwapUrl(explorerName: name, input: input.map(), providerId: provider) else {
            return nil
        }
        return BlockExplorerLink(name: url.name, link: url.url)
    }

    public func addressUrl(chain: Chain, address: String) -> BlockExplorerLink {
        let (name, explorer) = getExplorer(chain: chain)
        return makeLink(name: name, url: explorer.getAddressUrl(explorerName: name, address: address))!
    }

    public func tokenUrl(chain: Chain, address: String) -> BlockExplorerLink? {
        let (name, explorer) = getExplorer(chain: chain)
        return makeLink(name: name, url: explorer.getTokenUrl(explorerName: name, address: address))
    }

    public func nftUrl(chain: Chain, contractAddress: String, tokenId: String) -> BlockExplorerLink? {
        let (name, explorer) = getExplorer(chain: chain)
        return makeLink(name: name, url: explorer.getNftUrl(explorerName: name, contractAddress: contractAddress, tokenId: tokenId))
    }

    public func validatorUrl(chain: Chain, address: String) -> BlockExplorerLink? {
        let (name, explorer) = getExplorer(chain: chain)
        return makeLink(name: name, url: explorer.getValidatorUrl(explorerName: name, address: address))
    }

    // MARK: - Private

    private func getExplorer(chain: Chain) -> (String, Gemstone.Explorer) {
        (explorerNameOrDefault(chain: chain), Gemstone.Explorer(chain: chain.id))
    }

    private func makeLink(name: String, url: String?) -> BlockExplorerLink? {
        guard let url, let parsed = URL(string: url) else { return nil }
        return BlockExplorerLink(name: name, link: parsed.absoluteString)
    }

    private func explorerNameOrDefault(chain: Chain) -> String {
        let name = preferences.get(chain: chain)
        let explorers = Self.explorers(chain: chain)
        return explorers.first(where: { $0 == name }) ?? explorers.first!
    }
}

// MARK: - ExplorerStorable

extension ExplorerService: ExplorerPreferencesStorable {
    public func set(chain: Chain, name: String) {
        preferences.set(chain: chain, name: name)
    }

    public func get(chain: Chain) -> String? {
        preferences.get(chain: chain)
    }
}

// MARK: - ExplorerLinkFetchable

extension ExplorerService: ExplorerLinkFetchable {}
