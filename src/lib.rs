use validator::{Validate, ValidationErrors};
use yew::{html, Callback, Component, Context, Html, Properties};

#[cfg(feature = "derive")]
pub use macros::FormData;

pub trait FormData: Validate + Clone + PartialEq + 'static {
    type Update;
    type Callbacks;

    fn callbacks(update: Callback<Self::Update>) -> Self::Callbacks;
    fn update(&mut self, update: Self::Update);
}

pub trait FormDataView: FormData {
    fn view(&self, callbacks: Self::Callbacks, errors: Option<&ValidationErrors<Self>>, form_disabled: bool) -> Html;
}

pub struct Form<D: FormDataView> {
    data: D,
    error: Option<ValidationErrors<D>>,
}

#[derive(Properties, Clone, PartialEq)]
pub struct FormProps<D: FormDataView> {
    pub initial: D,
    pub on_submit: Callback<D>,
    pub submitting: bool,
    #[prop_or(false)]
    pub live_validation: bool,
}

pub enum FormMsg<D: FormDataView> {
    UpdateData(D::Update),
    Submit,
}

impl<D: FormDataView> Component for Form<D> {
    type Message = FormMsg<D>;
    type Properties = FormProps<D>;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            data: ctx.props().initial.clone(),
            error: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            FormMsg::UpdateData(update) => {
                self.data.update(update);
                if ctx.props().live_validation {
                    self.error = self.data.validate().err();
                }
                true
            }
            FormMsg::Submit => {
                self.error = self.data.validate().err();
                if self.error.is_none() {
                    ctx.props().on_submit.emit(self.data.clone());
                    false
                } else {
                    true
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let update = ctx.link().callback(FormMsg::UpdateData);
        let contents = self.data.view(D::callbacks(update), self.error.as_ref(), ctx.props().submitting);
        let submit = ctx.link().callback(|_| FormMsg::Submit);
        let submitting = ctx.props().submitting;
        html! {
            <form onsubmit={submit}>
                {contents}
                <input type="submit" value="Log in" disabled={submitting || self.error.is_some()} />
                if submitting {
                    <div class="spinner-container">
                        // Bootstrap example class, or use your custom CSS class
                        <div class="spinner-border" role="status">
                            <span class="visually-hidden">{ "Loading..." }</span>
                        </div>
                    </div>
                }
            </form>
        }
    }
}