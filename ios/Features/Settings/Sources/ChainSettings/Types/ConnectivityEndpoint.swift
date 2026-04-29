// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Primitives

struct ConnectivityEndpoint: Identifiable {
    let name: String
    let flag: String?
    let url: URL

    static let defaultEndpoints: [ConnectivityEndpoint] = [
        ConnectivityEndpoint(name: "API", flag: "🇺🇸", url: Constants.apiURL),
        ConnectivityEndpoint(name: Localized.Nodes.gemWalletNode, flag: "🇺🇸", url: Constants.nodesURL),
        ConnectivityEndpoint(name: Localized.Nodes.gemWalletNode, flag: "🇯🇵", url: Constants.nodesAsiaURL),
        ConnectivityEndpoint(name: Localized.Nodes.gemWalletNode, flag: "🇪🇺", url: Constants.nodesEuropeURL),
    ]

    var id: String {
        url.absoluteString
    }

    var title: String {
        [name, flag].compactMap(\.self).joined(separator: " ")
    }

    var host: String {
        url.host ?? url.absoluteString
    }
}
