#include <mupdf/fitz.h>

#define WRAP(name, ret_type, failure_val, call, ...) \
    ret_type mp_##name(fz_context *ctx, ##__VA_ARGS__) { \
        ret_type ret; \
        fz_try (ctx) { ret = call; } \
        fz_catch (ctx) { ret = failure_val; } \
        return ret; \
    }

WRAP(open_document, fz_document*, NULL, fz_open_document(ctx, path), char *path)
WRAP(open_document_with_stream, fz_document*, NULL, fz_open_document_with_stream(ctx, kind, stream), const char *kind, fz_stream *stream)
WRAP(load_page, fz_page*, NULL, fz_load_page(ctx, doc, pageno), fz_document *doc, int pageno)
WRAP(load_outline, fz_outline*, NULL, fz_load_outline(ctx, doc), fz_document *doc)
WRAP(count_pages, int, -1, fz_count_pages(ctx, doc), fz_document *doc)
WRAP(new_stext_page_from_page, fz_stext_page*, NULL, fz_new_stext_page_from_page(ctx, page, options), fz_page *page, fz_stext_options *options)
