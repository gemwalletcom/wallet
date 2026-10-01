// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import SwiftUI

public extension View {
    func alert<T: Sendable>(
        _ title: some StringProtocol,
        presenting data: Binding<T?>,
        sensoryFeedback: SensoryFeedback? = nil,
        @ViewBuilder actions: (T) -> some View,
        @ViewBuilder message: () -> some View = { EmptyView() },
    )
        -> some View
    {
        let isPresented = data.mappedToBool()

        return alert(
            title,
            isPresented: isPresented,
            presenting: data.wrappedValue,
            actions: { value in
                VStack {
                    actions(value)
                    Button(Localized.Common.cancel, role: .cancel) {
                        isPresented.wrappedValue = false
                    }
                }
            },
            message: { _ in
                message()
            },
        )
        .ifLet(sensoryFeedback) { view, value in
            view.sensoryFeedback(value, trigger: isPresented.wrappedValue) { $1 }
        }
    }
}
