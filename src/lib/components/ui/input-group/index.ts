import Root from './input-group.svelte';
import Addon, { type InputGroupAddonAlign } from './input-group-addon.svelte';
import Button, { type InputGroupButtonSize } from './input-group-button.svelte';
import Textarea from './input-group-textarea.svelte';

export type { InputGroupAddonAlign, InputGroupButtonSize };

export {
    Root,
    Addon,
    Button,
    Textarea,
    //
    Root as InputGroup,
    Addon as InputGroupAddon,
    Button as InputGroupButton,
    Textarea as InputGroupTextarea,
};
