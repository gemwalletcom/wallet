// Copyright (c). Gem Wallet. All rights reserved.

import Components
import SwiftUI

public extension View {
    func confirmationDialog<T: Sendable>(
        _ title: some StringProtocol,
        presenting data: Binding<T?>,
        sensoryFeedback: SensoryFeedback? = nil,
        @ViewBuilder actions: (T) -> some View,
        @ViewBuilder message: () -> some View = { EmptyView() },
    )
        -> some View
    {
        let isPresented = data.mappedToBool()
        let iPhone = UIDevice.current.userInterfaceIdiom == .phone

        return ifElse(iPhone) {
            $0.confirmationDialog(
                title,
                isPresented: isPresented,
                titleVisibility: .visible,
                presenting: data.wrappedValue,
                actions: actions,
                message: { _ in
                    message()
                },
            )
        } elseContent: {
            $0.alert(title, presenting: data, actions: actions, message: message)
        }
        .ifLet(sensoryFeedback) { view, value in
            view.sensoryFeedback(value, trigger: isPresented.wrappedValue) { $1 }
        }
    }
}
