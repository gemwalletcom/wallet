// Copyright (c). Gem Wallet. All rights reserved.

import Components
import SwiftUI

struct ChatwootWebScene: View {
    @State var model: ChatwootWebViewModel

    init(model: ChatwootWebViewModel) {
        _model = State(wrappedValue: model)
    }

    var body: some View {
        ZStack {
            ChatwootWebView(model: model)

            if model.isLoading {
                LoadingView()
            }
        }
    }
}
