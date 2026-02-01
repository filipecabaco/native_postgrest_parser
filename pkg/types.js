/**
 * Type-safe TypeScript definitions for PostgREST Parser
 *
 * This file provides strongly-typed interfaces that replace the `any` types
 * in the auto-generated wasm bindings, improving TypeScript developer experience.
 */
/**
 * Error thrown by the parser
 */
export class PostgRESTParserError extends Error {
    constructor(message, kind) {
        super(message);
        this.kind = kind;
        this.name = "PostgRESTParserError";
    }
}
