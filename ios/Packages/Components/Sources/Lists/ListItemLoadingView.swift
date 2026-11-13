// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

public struct ListItemLoadingView: View {
    public init() {}

    public var body: some View {
        LoadingView()
            .frame(maxWidth: .infinity)
            .listRowSeparator(.hidden)
            .listRowInsets(EdgeInsets())
            .listRowBackground(Color.clear)
    }
}

#Preview {
    List {
        Text("any text")
        Text("some text")
        Text("more text")
        Section {
            ListItemLoadingView()
        }
    }
    .listStyle(.insetGrouped)
}
