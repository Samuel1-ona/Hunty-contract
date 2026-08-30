import { Buffer } from "buffer";
export * from "@stellar/stellar-sdk";
export * as contract from "@stellar/stellar-sdk/contract";
export * as rpc from "@stellar/stellar-sdk/rpc";

if (typeof window !== "undefined") {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}

export * from "./types.js";
export * from "./events.js";
export * from "./errors.js";
export * from "./reward-types.js";
export * from "./client.js";
