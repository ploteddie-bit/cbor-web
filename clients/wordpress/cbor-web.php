<?php
/**
 * Plugin Name: CBOR-Web
 * Plugin URI: https://github.com/ploteddie-bit/cbor-web
 * Description: Publish machine-readable CBOR versions of your content for AI agents. Automatically generates .cbor files, serves a /.well-known/cbor-web manifest, and integrates token-gated access.
 * Version: 2.1.0
 * Author: ExploDev / Deltopide SL
 * License: MIT
 * License URI: https://opensource.org/licenses/MIT
 * Requires PHP: 8.0
 * Text Domain: cbor-web
 */

// ---------------------------------------------------------------------------
// CBOR Encoder — Deterministic (RFC 8949 §4.2)
// ---------------------------------------------------------------------------

if (!function_exists('cbor_encode_uint')) {
    function cbor_encode_uint(int $value): string {
        if ($value < 0) {
            // negative → nint (major 1)
            $n = -1 - $value;
            if ($n <= 23) return chr(0x20 | $n);
            if ($n <= 0xFF) return chr(0x38) . chr($n);
            if ($n <= 0xFFFF) return chr(0x39) . pack('n', $n);
            if ($n <= 0x7FFFFFFF) return chr(0x3A) . pack('N', $n);
            return chr(0x3B) . pack('J', $n);
        }
        if ($value <= 23) return chr($value);
        if ($value <= 0xFF) return chr(0x18) . chr($value);
        if ($value <= 0xFFFF) return chr(0x19) . pack('n', $value);
        if ($value <= 0xFFFFFFFF) return chr(0x1A) . pack('N', $value);
        return chr(0x1B) . pack('J', $value);
    }
}

if (!function_exists('cbor_encode_tstr')) {
    function cbor_encode_tstr(string $value): string {
        $len = strlen($value);
        $head = cbor_encode_uint($len);
        $head[0] = chr((ord($head[0]) & 0x1F) | 0x60);
        return $head . $value;
    }
}

if (!function_exists('cbor_encode_bstr')) {
    function cbor_encode_bstr(string $value): string {
        $len = strlen($value);
        $head = cbor_encode_uint($len);
        $head[0] = chr((ord($head[0]) & 0x1F) | 0x40);
        return $head . $value;
    }
}

if (!function_exists('cbor_encode_array')) {
    function cbor_encode_array(array $items): string {
        $len = count($items);
        $head = cbor_encode_uint($len);
        $head[0] = chr((ord($head[0]) & 0x1F) | 0x80);
        $out = $head;
        foreach ($items as $item) {
            $out .= $item;
        }
        return $out;
    }
}

if (!function_exists('cbor_encode_tag')) {
    function cbor_encode_tag(int $tag, string $inner): string {
        $head = cbor_encode_uint($tag);
        $head[0] = chr((ord($head[0]) & 0x1F) | 0xC0);
        return $head . $inner;
    }
}

if (!function_exists('cbor_encode_map')) {
    function cbor_encode_map(array $pairs): string {
        $len = count($pairs);
        $head = cbor_encode_uint($len);
        $head[0] = chr((ord($head[0]) & 0x1F) | 0xA0);
        // Deterministic key ordering: sort by encoded key (shortest first, then bytewise)
        uksort($pairs, function ($a, $b) {
            $ea = cbor_encode_key($a);
            $eb = cbor_encode_key($b);
            $la = strlen($ea);
            $lb = strlen($eb);
            if ($la !== $lb) return $la - $lb;
            return strcmp($ea, $eb);
        });
        $out = $head;
        foreach ($pairs as $k => $v) {
            $out .= cbor_encode_key($k) . $v;
        }
        return $out;
    }
}

if (!function_exists('cbor_encode_key')) {
    function cbor_encode_key($key): string {
        if (is_int($key)) return cbor_encode_uint($key);
        return cbor_encode_tstr((string)$key);
    }
}

if (!function_exists('cbor_encode_timestamp')) {
    function cbor_encode_timestamp(int $unix_seconds): string {
        return cbor_encode_tag(1, cbor_encode_uint($unix_seconds));
    }
}

if (!function_exists('cbor_encode_page_path')) {
    function cbor_encode_page_path(string $path): string {
        if ($path === '/') return '_index';
        $s = str_replace('_', '%5F', $path);
        $s = ltrim($s, '/');
        $s = str_replace('/', '_', $s);
        return $s;
    }
}

// ---------------------------------------------------------------------------
// HTML → CBOR Block Extractor
// ---------------------------------------------------------------------------

if (!function_exists('cbor_extract_blocks')) {
    function cbor_extract_blocks(string $html): array {
        $html = mb_convert_encoding($html, 'HTML-ENTITIES', 'UTF-8');
        libxml_use_internal_errors(true);
        $doc = new DOMDocument();
        $doc->loadHTML('<div>' . $html . '</div>');
        libxml_clear_errors();

        $blocks = [];
        $body = $doc->getElementsByTagName('div')->item(0);
        if (!$body) return $blocks;

        foreach ($body->childNodes as $node) {
            $block = cbor_node_to_block($node);
            if ($block) $blocks[] = $block;
        }
        return $blocks;
    }
}

if (!function_exists('cbor_node_to_block')) {
    function cbor_node_to_block(DOMNode $node): ?array {
        $name = strtolower($node->nodeName);
        $text = trim(cbor_strip_tags_inner($node));

        if ($text === '' && !in_array($name, ['img', 'figure', 'hr'])) return null;

        switch ($name) {
            case 'h1': return ['l' => 1, 't' => 'h', 'v' => $text];
            case 'h2': return ['l' => 2, 't' => 'h', 'v' => $text];
            case 'h3': return ['l' => 3, 't' => 'h', 'v' => $text];
            case 'h4': return ['l' => 4, 't' => 'h', 'v' => $text];
            case 'h5': return ['l' => 5, 't' => 'h', 'v' => $text];
            case 'h6': return ['l' => 6, 't' => 'h', 'v' => $text];
            case 'p':  return ['t' => 'p', 'v' => $text];
            case 'blockquote': return ['t' => 'q', 'v' => $text];
            case 'pre': return ['t' => 'code', 'v' => $text];
            case 'hr': return ['t' => 'sep'];
            case 'ul': return cbor_extract_list_items($node, 'ul');
            case 'ol': return cbor_extract_list_items($node, 'ol');
            case 'img':
                $src = '';
                $alt = '';
                if ($node instanceof DOMElement) {
                    $src = $node->getAttribute('src') ?: '';
                    $alt = $node->getAttribute('alt') ?: '';
                }
                return ['t' => 'img', 'src' => $src, 'alt' => $alt];
            case 'figure':
                foreach ($node->childNodes as $child) {
                    $b = cbor_node_to_block($child);
                    if ($b) return $b;
                }
                return null;
            case '#text':
                return $text !== '' ? ['t' => 'p', 'v' => $text] : null;
            default:
                return $text !== '' ? ['t' => 'p', 'v' => $text] : null;
        }
    }
}

if (!function_exists('cbor_extract_list_items')) {
    function cbor_extract_list_items(DOMNode $node, string $type): array {
        $items = [];
        foreach ($node->childNodes as $child) {
            if (strtolower($child->nodeName) === 'li') {
                $items[] = trim(cbor_strip_tags_inner($child));
            }
        }
        return ['t' => $type, 'v' => $items];
    }
}

if (!function_exists('cbor_strip_tags_inner')) {
    function cbor_strip_tags_inner(DOMNode $node): string {
        $text = '';
        foreach ($node->childNodes as $child) {
            if ($child->nodeType === XML_TEXT_NODE) {
                $text .= $child->textContent;
            } elseif ($child->nodeType === XML_ELEMENT_NODE) {
                $name = strtolower($child->nodeName);
                if (in_array($name, ['br', 'hr'])) {
                    $text .= "\n";
                } else {
                    $text .= cbor_strip_tags_inner($child);
                }
            }
        }
        return $text;
    }
}

// ---------------------------------------------------------------------------
// CBOR Page Encoder
// ---------------------------------------------------------------------------

if (!function_exists('cbor_encode_block')) {
    function cbor_encode_block(array $block): string {
        // Deterministic key order for content blocks: l(0x6C) < t(0x74) < v(0x76)
        // Keys that are 2 bytes each: 61 6C, 61 74, 61 76 — so bytewise: l, t, v
        $pairs = [];
        // Force explicit key encoding for deterministic sorting
        if (isset($block['l'])) $pairs['l'] = $block['l'];
        if (isset($block['t'])) $pairs['t'] = $block['t'];

        if ($block['t'] === 'ul' || $block['t'] === 'ol') {
            $items = [];
            foreach (($block['v'] ?? []) as $item) {
                $items[] = cbor_encode_tstr($item);
            }
            $pairs['v'] = cbor_encode_array($items);
        } elseif (isset($block['v'])) {
            $pairs['v'] = cbor_encode_tstr((string)$block['v']);
        }
        if (isset($block['src'])) $pairs['src'] = cbor_encode_tstr($block['src']);
        if (isset($block['alt'])) $pairs['alt'] = cbor_encode_tstr($block['alt']);

        return cbor_encode_map($pairs);
    }
}

if (!function_exists('cbor_generate_page')) {
    function cbor_generate_page(string $path, string $title, string $content, array $meta = []): string {
        $now = time();
        $blocks = cbor_extract_blocks($content);

        $identity = [
            'path' => cbor_encode_tstr($path),
            'lang' => cbor_encode_tstr($meta['lang'] ?? 'en'),
        ];
        if (!empty($meta['canon_url'])) {
            $identity['canon_url'] = cbor_encode_tstr($meta['canon_url']);
        }

        $metadata = [
            'title' => cbor_encode_tstr($title),
            'updated' => cbor_encode_timestamp($meta['updated'] ?? $now),
            'generated_at' => cbor_encode_timestamp($now),
        ];
        if (!empty($meta['description'])) {
            $metadata['description'] = cbor_encode_tstr($meta['description']);
        }

        $content_blocks = [];
        foreach ($blocks as $block) {
            $content_blocks[] = cbor_encode_block($block);
        }

        $page_map = cbor_encode_map([
            0 => cbor_encode_tstr('cbor-web-page'),
            1 => cbor_encode_uint(2),
            2 => cbor_encode_map($identity),
            3 => cbor_encode_map($metadata),
            4 => cbor_encode_array($content_blocks),
        ]);

        return cbor_encode_tag(55799, $page_map);
    }
}

if (!function_exists('cbor_generate_manifest')) {
    function cbor_generate_manifest(string $domain, array $pages, array $meta = []): string {
        $now = time();

        $site_meta = [
            'domain' => cbor_encode_tstr($domain),
            'name' => cbor_encode_tstr($meta['site_name'] ?? get_bloginfo('name')),
        ];
        if (!empty($meta['site_description'])) {
            $site_meta['description'] = cbor_encode_tstr($meta['site_description']);
        }

        $page_entries = [];
        foreach ($pages as $page) {
            $entry = [
                'path' => cbor_encode_tstr($page['path']),
                'title' => cbor_encode_tstr($page['title']),
            ];
            if (!empty($page['updated'])) {
                $entry['updated'] = cbor_encode_timestamp($page['updated']);
            }
            if (!empty($page['access'])) {
                $entry['access'] = cbor_encode_tstr($page['access']);
            }
            $page_entries[] = cbor_encode_map($entry);
        }

        $gen_meta = [
            'generator' => cbor_encode_tstr('cbor-web-wordpress/2.1'),
            'generated_at' => cbor_encode_timestamp($now),
        ];

        $manifest_map = cbor_encode_map([
            0 => cbor_encode_tstr('cbor-web-manifest'),
            1 => cbor_encode_uint(2),
            2 => cbor_encode_map($site_meta),
            3 => cbor_encode_array($page_entries),
            5 => cbor_encode_map($gen_meta),
        ]);

        return cbor_encode_tag(55799, $manifest_map);
    }
}

// ---------------------------------------------------------------------------
// Main Plugin Class
// ---------------------------------------------------------------------------

class CborWebPlugin {

    private static ?CborWebPlugin $instance = null;
    private array $options;
    private string $cbor_dir;

    public static function get_instance(): self {
        if (self::$instance === null) {
            self::$instance = new self();
        }
        return self::$instance;
    }

    private function __construct() {
        $this->options = get_option('cbor_web_options', [
            'enabled' => true,
            'default_access_level' => 'T0',
            'token_wallet' => '',
        ]);
        $this->cbor_dir = WP_CONTENT_DIR . '/cbor-web/';

        add_action('init', [$this, 'add_rewrite_rules']);
        add_filter('query_vars', [$this, 'add_query_vars']);
        add_action('template_redirect', [$this, 'handle_well_known']);
        add_action('admin_menu', [$this, 'add_settings_page']);
        add_action('admin_init', [$this, 'register_settings']);
        add_action('save_post', [$this, 'on_post_save'], 20, 3);
        add_action('admin_notices', [$this, 'admin_notice']);
        add_action('deleted_post', [$this, 'on_post_deleted']);
        add_filter('plugin_action_links_' . plugin_basename(__FILE__), [$this, 'plugin_action_links']);
    }

    // ---------------------------------------------------------------
    // Rewrite rules — serve /.well-known/cbor-web and page CBOR files
    // ---------------------------------------------------------------

    public function add_rewrite_rules(): void {
        add_rewrite_rule(
            '^\.well-known/cbor-web/pages/(.+?)\.cbor$',
            'index.php?cbor_web_page=$matches[1]',
            'top'
        );
        add_rewrite_rule(
            '^\.well-known/cbor-web/bundle$',
            'index.php?cbor_web_bundle=1',
            'top'
        );
        add_rewrite_rule(
            '^\.well-known/cbor-web$',
            'index.php?cbor_web_manifest=1',
            'top'
        );
        add_rewrite_rule(
            '^\.well-known/cbor-web-token$',
            'index.php?cbor_web_token=1',
            'top'
        );
    }

    public function add_query_vars(array $vars): array {
        $vars[] = 'cbor_web_manifest';
        $vars[] = 'cbor_web_page';
        $vars[] = 'cbor_web_bundle';
        $vars[] = 'cbor_web_token';
        return $vars;
    }

    public function handle_well_known(): void {
        if (!$this->options['enabled']) return;

        if (get_query_var('cbor_web_manifest')) {
            $this->serve_manifest();
        }
        if (get_query_var('cbor_web_bundle')) {
            $this->serve_bundle();
        }
        if (($page_file = get_query_var('cbor_web_page')) !== '') {
            $this->serve_page($page_file);
        }
        if (get_query_var('cbor_web_token')) {
            $this->serve_token_info();
        }
    }

    // ---------------------------------------------------------------
    // Endpoint handlers
    // ---------------------------------------------------------------

    private function serve_manifest(): void {
        $cbor = $this->maybe_generate_and_read('manifest');
        if ($cbor === null) {
            $this->generate_manifest_file();
            $cbor = $this->maybe_generate_and_read('manifest');
        }
        if ($cbor === null) {
            status_header(404);
            exit;
        }
        $this->output_cbor($cbor, 3600);
    }

    private function serve_page(string $filename): void {
        $filepath = $this->cbor_dir . 'pages/' . basename($filename);
        if (!file_exists($filepath)) {
            // Try on-demand generation from slug
            $this->generate_from_filename($filename);
        }
        if (!file_exists($filepath)) {
            status_header(404);
            exit;
        }
        $this->output_cbor(file_get_contents($filepath), 86400);
    }

    private function serve_bundle(): void {
        $cbor = $this->maybe_generate_and_read('bundle');
        if ($cbor === null) {
            $this->generate_bundle_file();
            $cbor = $this->maybe_generate_and_read('bundle');
        }
        if ($cbor === null) {
            status_header(404);
            exit;
        }
        $this->output_cbor($cbor, 3600);
    }

    private function serve_token_info(): void {
        header('Content-Type: text/plain');
        $wallet = $this->options['token_wallet'] ?? '';
        echo "CBOR-Web token gate\n";
        echo "Default access: " . ($this->options['default_access_level'] ?? 'T0') . "\n";
        if ($wallet) echo "Token wallet: $wallet\n";
        exit;
    }

    private function output_cbor(string $data, int $max_age): void {
        header('Content-Type: application/cbor');
        header('Content-Length: ' . strlen($data));
        header('Cache-Control: public, max-age=' . $max_age);
        header('Access-Control-Allow-Origin: *');
        echo $data;
        exit;
    }

    // ---------------------------------------------------------------
    // File generation
    // ---------------------------------------------------------------

    private function ensure_dirs(): void {
        if (!is_dir($this->cbor_dir)) {
            mkdir($this->cbor_dir, 0755, true);
        }
        if (!is_dir($this->cbor_dir . 'pages')) {
            mkdir($this->cbor_dir . 'pages', 0755, true);
        }
    }

    private function maybe_generate_and_read(string $type): ?string {
        $file = $this->cbor_dir . $type . '.cbor';
        if (!file_exists($file)) return null;
        return file_get_contents($file);
    }

    public function generate_all(): void {
        $this->ensure_dirs();
        $pages = $this->collect_pages();
        foreach ($pages as $page) {
            $this->generate_page_file($page['id'], $page['path'], $page['title'], $page['content'], $page);
        }
        $this->generate_manifest_file();
        $this->generate_bundle_file();

        $options = get_option('cbor_web_options');
        $options['pages_indexed'] = count($pages);
        $options['last_generation'] = time();
        update_option('cbor_web_options', $options);
        $this->options = $options;
    }

    private function collect_pages(): array {
        $posts = get_posts([
            'post_type' => ['page', 'post'],
            'post_status' => 'publish',
            'numberposts' => -1,
        ]);

        $pages = [];
        foreach ($posts as $post) {
            $path = wp_parse_url(get_permalink($post), PHP_URL_PATH) ?: '/';
            $pages[] = [
                'id' => $post->ID,
                'path' => $path,
                'title' => get_the_title($post),
                'content' => apply_filters('the_content', $post->post_content),
                'updated' => get_post_modified_time('U', true, $post),
                'description' => get_the_excerpt($post) ?: '',
            ];
        }
        return $pages;
    }

    private function generate_page_file(int $post_id, string $path, string $title, string $content, array $meta): void {
        $this->ensure_dirs();
        $filename = cbor_encode_page_path($path);
        $cbor = cbor_generate_page($path, $title, $content, [
            'lang' => get_locale() ?: 'en',
            'updated' => $meta['updated'] ?? time(),
            'description' => $meta['description'] ?? '',
            'canon_url' => get_permalink($post_id),
        ]);
        file_put_contents($this->cbor_dir . 'pages/' . $filename . '.cbor', $cbor);
    }

    private function generate_manifest_file(): void {
        $this->ensure_dirs();
        $pages = $this->collect_pages();
        $manifest_pages = [];
        foreach ($pages as $page) {
            $manifest_pages[] = [
                'path' => $page['path'],
                'title' => $page['title'],
                'updated' => $page['updated'],
                'access' => $this->options['default_access_level'] ?? 'T0',
            ];
        }
        $cbor = cbor_generate_manifest(
            wp_parse_url(get_site_url(), PHP_URL_HOST) ?: 'localhost',
            $manifest_pages,
            [
                'site_name' => get_bloginfo('name'),
                'site_description' => get_bloginfo('description'),
            ]
        );
        file_put_contents($this->cbor_dir . 'manifest.cbor', $cbor);
    }

    private function generate_bundle_file(): void {
        $this->ensure_dirs();
        // Bundle: array of [manifest, page1, page2, ...]
        $parts = [];
        $manifest = $this->maybe_generate_and_read('manifest');
        if (!$manifest) {
            $this->generate_manifest_file();
            $manifest = $this->maybe_generate_and_read('manifest');
        }
        if ($manifest) $parts[] = $manifest;

        $pages_dir = $this->cbor_dir . 'pages/';
        if (is_dir($pages_dir)) {
            foreach (scandir($pages_dir) as $file) {
                if (str_ends_with($file, '.cbor')) {
                    $parts[] = file_get_contents($pages_dir . $file);
                }
            }
        }

        $bundle = cbor_encode_tag(55799, cbor_encode_array($parts));
        file_put_contents($this->cbor_dir . 'bundle.cbor', $bundle);
    }

    private function generate_from_filename(string $filename): void {
        // Reverse the path encoding to find the WordPress page
        $path = cbor_decode_page_path_from_filename($filename);
        $post_id = url_to_postid(home_url($path));
        if ($post_id) {
            $post = get_post($post_id);
            if ($post && $post->post_status === 'publish') {
                $this->generate_page_file(
                    $post->ID,
                    $path,
                    get_the_title($post),
                    apply_filters('the_content', $post->post_content),
                    [
                        'updated' => get_post_modified_time('U', true, $post),
                        'description' => get_the_excerpt($post) ?: '',
                    ]
                );
            }
        }
    }

    // ---------------------------------------------------------------
    // WordPress hooks
    // ---------------------------------------------------------------

    public function on_post_save(int $post_id, WP_Post $post, bool $update): void {
        if (!$this->options['enabled']) return;
        if (wp_is_post_revision($post_id) || wp_is_post_autosave($post_id)) return;
        if (!in_array($post->post_type, ['page', 'post'])) return;
        if ($post->post_status !== 'publish') {
            $this->on_post_deleted($post_id);
            return;
        }

        $path = wp_parse_url(get_permalink($post_id), PHP_URL_PATH) ?: '/';
        $this->generate_page_file(
            $post_id,
            $path,
            get_the_title($post),
            apply_filters('the_content', $post->post_content),
            [
                'updated' => get_post_modified_time('U', true, $post),
                'description' => get_the_excerpt($post) ?: '',
            ]
        );
        $this->generate_manifest_file();
        $this->generate_bundle_file();

        // Update stats
        $pages = $this->collect_pages();
        $options = get_option('cbor_web_options');
        $options['pages_indexed'] = count($pages);
        $options['last_generation'] = time();
        update_option('cbor_web_options', $options);
        $this->options = $options;
    }

    public function on_post_deleted(int $post_id): void {
        $path = wp_parse_url(get_permalink($post_id), PHP_URL_PATH) ?: '/';
        $filename = cbor_encode_page_path($path);
        $filepath = $this->cbor_dir . 'pages/' . $filename . '.cbor';
        if (file_exists($filepath)) {
            unlink($filepath);
        }
        $this->generate_manifest_file();
        $this->generate_bundle_file();

        $options = get_option('cbor_web_options');
        $options['pages_indexed'] = count($this->collect_pages());
        $options['last_generation'] = time();
        update_option('cbor_web_options', $options);
        $this->options = $options;
    }

    // ---------------------------------------------------------------
    // Settings page
    // ---------------------------------------------------------------

    public function add_settings_page(): void {
        add_options_page(
            'CBOR-Web',
            'CBOR-Web',
            'manage_options',
            'cbor-web',
            [$this, 'render_settings_page']
        );
    }

    public function register_settings(): void {
        register_setting('cbor_web_settings', 'cbor_web_options', [
            'type' => 'array',
            'sanitize_callback' => [$this, 'sanitize_options'],
            'default' => [
                'enabled' => true,
                'default_access_level' => 'T0',
                'token_wallet' => '',
            ],
        ]);

        add_settings_section('cbor_web_main', 'CBOR-Web Configuration', null, 'cbor-web');

        add_settings_field('cbor_web_enabled', 'Enable CBOR-Web', function () {
            $checked = checked($this->options['enabled'] ?? true, true, false);
            echo '<label><input type="checkbox" name="cbor_web_options[enabled]" value="1" ' . $checked . '> ';
            echo 'Serve CBOR-Web manifest and page files</label>';
        }, 'cbor-web', 'cbor_web_main');

        add_settings_field('cbor_web_access', 'Default Access Level', function () {
            $level = $this->options['default_access_level'] ?? 'T0';
            $opts = ['T0' => 'T0 — Public (no token required)', 'T1' => 'T1 — Token-preferred', 'T2' => 'T2 — Token-required'];
            echo '<select name="cbor_web_options[default_access_level]">';
            foreach ($opts as $val => $label) {
                echo '<option value="' . esc_attr($val) . '" ' . selected($level, $val, false) . '>' . esc_html($label) . '</option>';
            }
            echo '</select>';
            echo '<p class="description">T0 = public content visible to all agents. T1/T2 = token-gated content.</p>';
        }, 'cbor-web', 'cbor_web_main');

        add_settings_field('cbor_web_wallet', 'Token Wallet Address', function () {
            $wallet = $this->options['token_wallet'] ?? '';
            echo '<input type="text" name="cbor_web_options[token_wallet]" value="' . esc_attr($wallet) . '" class="regular-text" placeholder="0x...">';
            echo '<p class="description">Ethereum wallet address for token-gated CBOR-Web content access.</p>';
        }, 'cbor-web', 'cbor_web_main');
    }

    public function sanitize_options(array $input): array {
        $sanitized = [];
        $sanitized['enabled'] = !empty($input['enabled']);
        $sanitized['default_access_level'] = in_array($input['default_access_level'] ?? '', ['T0', 'T1', 'T2'])
            ? $input['default_access_level'] : 'T0';
        $sanitized['token_wallet'] = sanitize_text_field($input['token_wallet'] ?? '');
        // Preserve stats
        $old = get_option('cbor_web_options');
        $sanitized['pages_indexed'] = $old['pages_indexed'] ?? 0;
        $sanitized['last_generation'] = $old['last_generation'] ?? 0;
        return $sanitized;
    }

    public function render_settings_page(): void {
        // Handle manual regeneration
        if (isset($_POST['cbor_web_regenerate']) && check_admin_referer('cbor_web_regenerate')) {
            $this->generate_all();
            echo '<div class="notice notice-success is-dismissible"><p>CBOR-Web manifest and pages regenerated successfully.</p></div>';
        }

        // Flush rewrite rules if requested
        if (isset($_POST['cbor_web_flush_rewrite']) && check_admin_referer('cbor_web_flush_rewrite')) {
            flush_rewrite_rules();
            echo '<div class="notice notice-success is-dismissible"><p>Rewrite rules flushed.</p></div>';
        }

        echo '<div class="wrap">';
        echo '<h1>CBOR-Web Settings</h1>';

        // Stats card
        $indexed = $this->options['pages_indexed'] ?? 0;
        $last_gen = $this->options['last_generation'] ?? 0;
        echo '<div class="card" style="max-width:600px;padding:12px 20px;margin:20px 0;background:#fff;border:1px solid #c3c4c7;">';
        echo '<h3>Status</h3>';
        echo '<p><strong>Pages indexed:</strong> ' . (int)$indexed . '</p>';
        echo '<p><strong>Last generation:</strong> ' . ($last_gen ? wp_date('Y-m-d H:i:s', $last_gen) : 'Never') . '</p>';
        echo '<p><strong>Storage:</strong> <code>' . esc_html($this->cbor_dir) . '</code></p>';
        echo '</div>';

        echo '<form method="post" action="options.php">';
        settings_fields('cbor_web_settings');
        do_settings_sections('cbor-web');
        submit_button('Save Settings');
        echo '</form>';

        // Manual regenerate
        echo '<hr style="margin:20px 0">';
        echo '<h2>Manual Regeneration</h2>';
        echo '<p>Regenerate all CBOR files from current WordPress content.</p>';
        echo '<form method="post">';
        wp_nonce_field('cbor_web_regenerate');
        echo '<input type="submit" name="cbor_web_regenerate" class="button button-secondary" value="Regenerate All CBOR Files">';
        echo '</form>';

        // Flush rewrites
        echo '<form method="post" style="margin-top:12px">';
        wp_nonce_field('cbor_web_flush_rewrite');
        echo '<input type="submit" name="cbor_web_flush_rewrite" class="button button-secondary" value="Flush Rewrite Rules">';
        echo '</form>';

        // Endpoints info
        echo '<hr style="margin:20px 0">';
        echo '<h2>Endpoints</h2>';
        echo '<table class="widefat fixed striped" style="max-width:700px">';
        echo '<thead><tr><th>URL</th><th>Content</th></tr></thead>';
        echo '<tbody>';
        echo '<tr><td><code>' . esc_html(home_url('/.well-known/cbor-web')) . '</code></td><td>Manifest</td></tr>';
        echo '<tr><td><code>' . esc_html(home_url('/.well-known/cbor-web/pages/{file}.cbor')) . '</code></td><td>Page CBOR</td></tr>';
        echo '<tr><td><code>' . esc_html(home_url('/.well-known/cbor-web/bundle')) . '</code></td><td>Bundle (all pages)</td></tr>';
        echo '<tr><td><code>' . esc_html(home_url('/.well-known/cbor-web-token')) . '</code></td><td>Token info (plain text)</td></tr>';
        echo '</tbody></table>';

        echo '</div>';
    }

    // ---------------------------------------------------------------
    // Admin notice
    // ---------------------------------------------------------------

    public function admin_notice(): void {
        $screen = get_current_screen();
        if (!$screen) return;
        if (!in_array($screen->id, ['dashboard', 'settings_page_cbor-web', 'plugins'])) return;

        $indexed = $this->options['pages_indexed'] ?? 0;
        $last_gen = $this->options['last_generation'] ?? 0;
        $enabled = $this->options['enabled'] ?? false;

        if ($enabled && $indexed > 0 && $last_gen) {
            $time_ago = human_time_diff($last_gen, time());
            echo '<div class="notice notice-info is-dismissible">';
            echo '<p><strong>CBOR-Web:</strong> ' . (int)$indexed . ' pages indexed. Last generation: ' . esc_html($time_ago) . ' ago. ';
            echo '<a href="' . esc_url(admin_url('options-general.php?page=cbor-web')) . '">Manage settings</a></p>';
            echo '</div>';
        } elseif ($enabled && $indexed === 0) {
            echo '<div class="notice notice-warning is-dismissible">';
            echo '<p><strong>CBOR-Web:</strong> No pages indexed yet. Save a page/post to trigger generation, or ';
            echo '<a href="' . esc_url(admin_url('options-general.php?page=cbor-web')) . '">regenerate manually</a>.</p>';
            echo '</div>';
        }
    }

    // ---------------------------------------------------------------
    // Plugin action links
    // ---------------------------------------------------------------

    public function plugin_action_links(array $links): array {
        $links[] = '<a href="' . esc_url(admin_url('options-general.php?page=cbor-web')) . '">Settings</a>';
        return $links;
    }
}

// ---------------------------------------------------------------------------
// Activation / Deactivation hooks
// ---------------------------------------------------------------------------

function cbor_web_activate(): void {
    $instance = CborWebPlugin::get_instance();
    // Ensure directory exists and hook flags
    $dir = WP_CONTENT_DIR . '/cbor-web/';
    if (!is_dir($dir)) mkdir($dir, 0755, true);
    if (!is_dir($dir . 'pages')) mkdir($dir . 'pages', 0755, true);
    flush_rewrite_rules();
}

function cbor_web_deactivate(): void {
    flush_rewrite_rules();
}

function cbor_web_uninstall(): void {
    delete_option('cbor_web_options');
    // Clean up files
    $dir = WP_CONTENT_DIR . '/cbor-web/';
    if (is_dir($dir)) {
        array_map('unlink', glob($dir . 'pages/*.cbor') ?: []);
        @rmdir($dir . 'pages');
        array_map('unlink', glob($dir . '*.cbor') ?: []);
        @rmdir($dir);
    }
}

register_activation_hook(__FILE__, 'cbor_web_activate');
register_deactivation_hook(__FILE__, 'cbor_web_deactivate');
register_uninstall_hook(__FILE__, 'cbor_web_uninstall');

// Helper — reverse path encoding
if (!function_exists('cbor_decode_page_path_from_filename')) {
    function cbor_decode_page_path_from_filename(string $filename): string {
        $s = str_replace('.cbor', '', $filename);
        if ($s === '_index') return '/';
        $s = str_replace('_', '/', $s);
        $s = rawurldecode($s);
        return '/' . $s;
    }
}

// Bootstrap
CborWebPlugin::get_instance();
