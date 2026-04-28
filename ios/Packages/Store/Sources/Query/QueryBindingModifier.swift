// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI

struct QueryBindingModifier: ViewModifier {
    @Environment(\.database) private var database

    let queries: [any BindableQuery]

    func body(content: Content) -> some View {
        content
            .onChange(of: queries.map { ObjectIdentifier($0) }, initial: true) {
                for query in queries {
                    query.bind(dbQueue: database.dbQueue)
                }
            }
    }
}

public extension View {
    func bindQuery(_ queries: any BindableQuery...) -> some View {
        modifier(QueryBindingModifier(queries: queries))
    }
}
