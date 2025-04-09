import { StyleSheet, Text, TouchableOpacity, View } from "react-native";
import { ExampleFfi, type RustEventHandler } from "../../src";
import { useState } from "react";

const exampleApi = new ExampleFfi();

export default function App() {
  const [val, setVal] = useState(0);

  class EventHandler implements RustEventHandler {
    onUpdate(update: number): void {
      console.log(`updating to ${update}`);
      setVal(update);
    }
  }

  const eventHandler = new EventHandler();
  exampleApi.replaceEventHandler(eventHandler);
  return (
    <View style={styles.container}>
      <TouchableOpacity
        onPress={() => {
          exampleApi.startIdempotent();
        }}
      >
        <Text>Start {val > 0 ? `${val}` : ""}</Text>
      </TouchableOpacity>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    alignItems: "center",
    justifyContent: "center",
  },
  box: {
    width: 60,
    height: 60,
    marginVertical: 20,
  },
});
